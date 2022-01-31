#define TRANSMITTANCE_INTEGRAL_SAMPLES 1000

#define TRANSMITTANCE_WIDTH 256
#define TRANSMITTANCE_HEIGHT 64

#define IRRADIANCE_WIDTH 64
#define IRRADIANCE_HEIGHT 16

#define INSCATTER_MU_S_SIZE 32
#define INSCATTER_NU_SIZE 8
#define INSCATTER_MU_SIZE 128
#define INSCATTER_R_SIZE 32

#define INSCATTER_INTEGRAL_SAMPLES 1000

cbuffer AtmosphericConstants : register(b0) {
    float3 beta_rayleigh;
    uint num_scattering;
    float3 wave_lengths;
    float mu_s_min;
    float3 solar_irradiance;
    float atmos_bottom;
    float atmos_top;
    float Hm;
    float Hr;
};

StructuredBuffer<float3> Transmittance : register(t0);
RWStructuredBuffer<float3> DeltaInScatterRayleigh : register(u0);
RWStructuredBuffer<float3> DeltaInScatterMie : register(u1);

float DistanceToAtmosTop(float r, float mu) {

    float discriminant = (r*r * ((mu*mu) - 1.0)) + (atmos_top * atmos_top);

    return max(0.0, (-r * mu) + sqrt(max(0.0, discriminant)));
}

float DistanceToAtmosBottom(float r, float mu) {
    return max(0.0, (-r * mu) - sqrt(max(r * r * (mu * mu - 1.0) + atmos_bottom * atmos_bottom, 0.0)));
}

float DistanceToAtmos(float r, float mu, bool intersects_ground) {
    if (intersects_ground) {
        return DistanceToAtmosBottom(r, mu);
    }

    return DistanceToAtmosTop(r, mu);
}

bool ViewIntersectsGround(float r, float mu) {
    return mu < 0.0 && r*r * (mu*mu - 1.0) + atmos_bottom * atmos_bottom >= 0.0;
}

float3 BetaMieScattering(float r, float mu) {
    return float3(4e-3, 4e-3, 4e-3);
}

float3 BetaMieExtinction(float r, float mu) {
    return BetaMieScattering(r,mu) / 0.9;
}

float2 GetTransmittanceUVFromRMu(float r, float mu) {
    float H = sqrt(atmos_top * atmos_top - atmos_bottom * atmos_bottom);
    float rho = sqrt(max(0.0, r * r - atmos_bottom * atmos_bottom));

    float d = DistanceToAtmosTop(r, mu);
    float d_min = atmos_top - r;
    float d_max = rho + H;

    float x_mu = (d - d_min) / (d_max - d_min);
    float x_r = rho / H;

    return float2(x_mu, x_r);
}

float3 GetTransmittance(float r, float mu) {

    float2 uv = GetTransmittanceUVFromRMu(r, mu);
    uint2 xy = uint2(TRANSMITTANCE_WIDTH * uv.x, TRANSMITTANCE_HEIGHT * uv.y);
    uint index = TRANSMITTANCE_WIDTH * xy.y + xy.x;

    return Transmittance[index];
}

float3 GetTransmittance(float r, float mu, float dist, bool intersects_ground) {
   
    float r_y = sqrt(r*r + dist*dist + 2.0*r*mu*dist);
    float mu_y = clamp((r*mu + dist) / r_y, -1.0, 1.0);

    if (intersects_ground) {
        return min(
            1.0,
            GetTransmittance(r_y, -mu_y) / GetTransmittance(r, -mu) // Negative because we're facing down, but want to look up
        );
    }
    else {
        return min(
            1.0,
            GetTransmittance(r, mu) / GetTransmittance(r_y, mu_y)
        );
    }
}

float2 RMusFromUV(float2 uv) {
    float mu_s = uv.x;
    float r = uv.y;

    r = atmos_bottom + (r * (atmos_top - atmos_bottom));
    mu_s = clamp(2.0 * mu_s - 1.0, -1.0, 1.0);


    return float2(r, mu_s);
}

#define mod(x, y) (x - y * floor(x / y))

float4 GetRMuMuSNuFromUVWZ(float4 uvwz, out bool intersects_ground) {
    float H = sqrt(atmos_top * atmos_top - atmos_bottom * atmos_bottom);
    float rho = H * uvwz.w;
    float r = sqrt(rho * rho + atmos_bottom * atmos_bottom);

    float mu = 0.0;
    if (uvwz.z < 0.5) {
        float d_min = r - atmos_bottom;
        float d_max = rho;
        float d = d_min + (d_max - d_min) * (1.0 - 2.0 * uvwz.z);
        mu = d == 0.0 ? -1.0 : clamp(-(rho * rho + d*d) / (2.0 * r * d), -1.0, 1.0);
        
        intersects_ground = true;
    }
    else {
        float d_min = atmos_top - r;
        float d_max = rho + H;

        // Check this part
        float d = d_min + (d_max - d_min) * (2.0 * uvwz.z - 1.0);

        mu = d == 0.0 ? 1.0 : clamp((H*H - rho*rho - d*d) / (2.0 * r * d), -1.0, 1.0);
        
        intersects_ground = false;
    }

    float x_mu_s = uvwz.y;
    float d_min = atmos_top - atmos_bottom;
    float d_max = H;
    float D = DistanceToAtmosTop(atmos_bottom, mu_s_min);

    float A = (D - d_min) / (d_max - d_min);
    float a = (A - x_mu_s * A) / (1.0 + x_mu_s * A);
    float d = d_min + min(a,A) * (d_max - d_min);
    float mu_s = d == 0.0 ? 1.0 : clamp((H*H - d*d) / (2.0 * atmos_bottom * d), -1.0, 1.0);

    float nu = clamp(uvwz.x * 2.0 - 1.0, -1.0, 1.0);

    return float4(r, mu, mu_s, nu);
}

void InScatterAtPoint(float r, float mu, float mu_s, float nu, float dist, bool intersects_ground, out float3 rayleigh, out float3 mie)
{
    rayleigh = float3(0.0, 0.0, 0.0);
    mie = float3(0.0, 0.0, 0.0);

    // Single scattering, so only direct from sun. Don't need to calculate integral

    float r_y = sqrt(r*r + dist*dist + 2.0*r*mu*dist);
    float mu_s_y = (nu*dist + mu_s*r) / r_y;

    r_y = max(atmos_bottom, r_y);

    if (mu_s_y >= -sqrt(1.0 - ((atmos_bottom*atmos_bottom) / (r_y * r_y)))) {
        float3 transmittance = GetTransmittance(r, mu, dist, intersects_ground) * GetTransmittance(r_y, mu_s_y);
        rayleigh = exp(-(r_y - atmos_bottom) / Hr) * transmittance;
        mie = exp(-(r_y - atmos_bottom) / Hm) * transmittance;
    }

}

void CalculateSingleInScatter(float r, float mu, float mu_s, float nu, bool intersects_ground, out float3 rayleigh, out float3 mie)
{
    rayleigh = float3(0.0, 0.0, 0.0);
    mie = float3(0.0, 0.0, 0.0);

    float dx = DistanceToAtmos(r, mu, intersects_ground) / INSCATTER_INTEGRAL_SAMPLES;

    float3 rayleigh_sum = float3(0.0, 0.0, 0.0);
    float3 mie_sum = float3(0.0, 0.0, 0.0);

    for (int i = 0; i <= INSCATTER_INTEGRAL_SAMPLES; i++) {
        float d_i = i * dx;

        float3 rayleigh_i;
        float3 mie_i;

        InScatterAtPoint(r, mu, mu_s, nu, d_i, intersects_ground, rayleigh_i, mie_i);

        float weight_i = (i == 0 || i == INSCATTER_INTEGRAL_SAMPLES) ? 0.5 : 1.0;
        rayleigh_sum += rayleigh_i * weight_i;
        mie_sum += mie_i * weight_i;
    }

    rayleigh = rayleigh_sum * dx * solar_irradiance * beta_rayleigh;
    mie = mie_sum * dx * solar_irradiance * BetaMieScattering(r, mu);

}

float4 GetRMuMuSNuFromUVW(float3 uvw, out bool intersects_ground) {
    const float4 texture_sizes = float4(INSCATTER_NU_SIZE - 1, INSCATTER_MU_S_SIZE - 1, INSCATTER_MU_SIZE - 1, INSCATTER_R_SIZE - 1);

    float frag_nu = uvw.x / INSCATTER_MU_S_SIZE;

    float frag_mu_s = mod(uvw.x, INSCATTER_MU_S_SIZE);

    float4 uvwz = float4(frag_nu, frag_mu_s, uvw.y, uvw.z) / texture_sizes;

    float4 rMuMusNu = GetRMuMuSNuFromUVWZ(uvwz, intersects_ground);

    float r    = rMuMusNu.x;
    float mu   = rMuMusNu.y;
    float mu_s = rMuMusNu.z;
    float nu   = rMuMusNu.w;

    nu = clamp(nu, mu * mu_s - sqrt((1.0 - mu * mu) * (1.0 - mu_s * mu_s)), mu * mu_s + sqrt((1.0 - mu * mu) * (1.0 - mu_s * mu_s)));

    return float4(r, mu, mu_s, nu);
}


[numthreads(32, 1, 1)]
void main (uint3 DTid: SV_DispatchThreadId) {

    bool intersects_ground = false;
    float4 rMuMusNu = GetRMuMuSNuFromUVW(DTid.xyz, intersects_ground);

    float3 rayleigh;
    float3 mie;

    CalculateSingleInScatter(rMuMusNu.x, rMuMusNu.y, rMuMusNu.z, rMuMusNu.w, intersects_ground, rayleigh, mie);

    uint index = (DTid.z * INSCATTER_MU_S_SIZE * INSCATTER_MU_SIZE * INSCATTER_NU_SIZE) + (DTid.y * INSCATTER_MU_S_SIZE * INSCATTER_NU_SIZE + DTid.x);
    DeltaInScatterRayleigh[index] = rayleigh;
    DeltaInScatterMie[index] = mie;
}