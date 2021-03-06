
#define TRANSMITTANCE_INTEGRAL_SAMPLES 1000

struct AtmosphericConstants {
    float atmos_bottom;
    float atmos_top;
    float Hm;
    float Hr;
    float3 beta_rayleigh;
    float3 wave_lengths;
    uint num_scattering;
};

StructuredBuffer<AtmosphericConstants> Constants : register(t0);
RWTexture2D<float4> Transmittance : register(u0);

float DistanceToAtmosTop(float r, float mu) {
    return (-r * mu) + max(sqrt(r * r * (mu * mu - 1.0) + Constants[0].atmos_top * Constants[0].atmos_top), 0.0);
}

float DistanceToAtmosBottom(float r, float mu) {
    return (-r * mu) - max(sqrt(r * r * (mu * mu - 1.0) + Constants[0].atmos_bottom * Constants[0].atmos_bottom), 0.0);
}

bool ViewIntersectsGround(float r, float mu) {
    return mu < 0.0 && r*r * (mu*mu - 1.0) + Constants[0].atmos_bottom * Constants[0].atmos_bottom >= 0.0;
}

float3 BetaMieScattering(float r, float mu) {
    return float3(4e-3, 4e-3, 4e-3);
}

float3 BetaMieExtinction(float r, float mu) {
    return BetaMieScattering(r,mu) / 0.9;
}

float DensityAlongView(float scale_height, float r, float mu) {

    bool ray_below_horizon = mu < -sqrt(1.0 - ((Constants[0].atmos_bottom * Constants[0].atmos_bottom) / (r * r)));
    if (ray_below_horizon) {
        return 1e9;
    }

    float total_density = 0.0;
    float dx = DistanceToAtmosTop(r, mu) / TRANSMITTANCE_INTEGRAL_SAMPLES;

    float y_j = exp(- (r - Constants[0].atmos_bottom) / scale_height);

    for (int i = 1; i <= TRANSMITTANCE_INTEGRAL_SAMPLES; i++) {
        float x_i = float(i) * dx;
        float r_i = sqrt(r * r + x_i * x_i + 2.0 * x_i * r * mu);
        float y_i = exp(-(r_i - Constants[0].atmos_bottom) / scale_height);
        total_density += (y_j + y_i) / 2.0 * dx;

        y_j = y_i;
    }

    return total_density;

}

float2 RMuFromUV(float2 uv) {
    float mu = uv.x;
    float r = uv.y;

    float distance_to_top_for_horizontal = sqrt(Constants[0].atmos_top * Constants[0].atmos_top - Constants[0].atmos_bottom * Constants[0].atmos_bottom);

    float rho = distance_to_top_for_horizontal * r;

    r = sqrt(max(rho * rho + Constants[0].atmos_bottom * Constants[0].atmos_bottom, 0.0));

    float d_min = Constants[0].atmos_top - r;
    float d_max = rho + distance_to_top_for_horizontal;

    float d = d_min + mu * (d_max - d_min);
    mu = d == 0.0 ? 1.0 : (distance_to_top_for_horizontal * distance_to_top_for_horizontal - rho * rho - d*d) / (2.0 * r * d);

    mu = clamp(mu, -1.0, 1.0);

    return float2(r, mu);
}

[numthreads(32, 1, 1)]
void main (uint3 DTid: SV_DispatchThreadId) {
    float2 uv = float2(DTid.x / 255.0, DTid.y / 63.0);


    float2 r_mu = RMuFromUV(uv);
    float r = r_mu.x;
    float mu = r_mu.y;

    float dist_to_top = DistanceToAtmosTop(r, mu);
    bool intersects_ground = ViewIntersectsGround(r, mu);
    float3 t = ComputeTransmittanceToAtmosTop(r, mu);

    Transmittance[DTid.xy] = float4(t.xyz, dist_to_top);
}