#define TRANSMITTANCE_INTEGRAL_SAMPLES 1000

#define TRANSMITTANCE_WIDTH 256
#define TRANSMITTANCE_HEIGHT 64

#define IRRADIANCE_WIDTH 64
#define IRRADIANCE_HEIGHT 16

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
RWStructuredBuffer<float3> DeltaIrradiance : register(u0);

float DistanceToAtmosTop(float r, float mu) {

    float discriminant = (r*r * ((mu*mu) - 1.0)) + (atmos_top * atmos_top);

    return max(0.0, (-r * mu) + sqrt(max(0.0, discriminant)));
}

float DistanceToAtmosBottom(float r, float mu) {
    return max(0.0, (-r * mu) - sqrt(max(r * r * (mu * mu - 1.0) + atmos_bottom * atmos_bottom, 0.0)));
}

bool ViewIntersectsGround(float r, float mu) {
    return mu < 0.0 && r*r * (mu*mu - 1.0) + atmos_bottom * atmos_bottom >= 0.0;
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

float3 ComputeIrradiance(float r, float mu_s) {
    float3 attenuation = GetTransmittance(r, mu_s);
    return attenuation * saturate(mu_s) * solar_irradiance;
}

float2 RMusFromUV(float2 uv) {
    float mu_s = uv.x;
    float r = uv.y;

    r = atmos_bottom + (r * (atmos_top - atmos_bottom));
    mu_s = clamp(2.0 * mu_s - 1.0, -1.0, 1.0);


    return float2(r, mu_s);
}

[numthreads(32, 1, 1)]
void main (uint3 DTid: SV_DispatchThreadId) {
    float2 uv = float2(DTid.x / (IRRADIANCE_WIDTH - 1.0), DTid.y / (IRRADIANCE_HEIGHT - 1.0));


    float2 r_mu = RMusFromUV(uv);
    float r = r_mu.x;
    float mu_s = r_mu.y;

    float3 irradiance = ComputeIrradiance(r, mu_s);

    uint index = (DTid.y * IRRADIANCE_WIDTH + DTid.x);
    DeltaIrradiance[index] = irradiance;
}