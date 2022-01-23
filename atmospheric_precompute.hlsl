
struct AtmosphericConstants {
    float atmos_bottom;
    float atmos_top;
    float3 beta_rayleigh;
    float3 wave_lengths;
    uint num_scattering;
};

StructuredBuffer<AtmosphericConstants> Constants : register(t0);
RWTexture2D<float4> Transmittance : register(u0);


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

    uint index = DTid.y * 64 + DTid.x;


    float2 uv = float2(DTid.x / 63.0, DTid.y / 255.0);


    float2 r_mu = RMuFromUV(uv);




    Transmittance[DTid.xy] = float4(uv.x, uv.y, r_mu.x, r_mu.y);


}