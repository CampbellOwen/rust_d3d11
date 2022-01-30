
Texture2D pos : register(t0);
Texture2D albedo : register(t1);
Texture2D normal : register(t2);

Texture2D transmittance : register(t3);
Texture2D irradiance : register(t4);

SamplerState Sampler;

float GetUVFromUnitRange(float x, int texture_size) {
  return 0.5 / (texture_size) + x * (1.0 - 1.0 / (texture_size));
}
float2 GetIrradianceUVFromRMuS(float r, float mu_s) {
    float y = (r - 6360.0) / (6420.0 - 6360.0);
    float x = mu_s * 0.5 + 0.5;

    return float2(GetUVFromUnitRange(x, 64), GetUVFromUnitRange(y,  16));
}

float4 main(float4 position : SV_POSITION, float2 uv: TEXCOORD) : SV_TARGET {


    float4 n = normal.Sample(Sampler, uv);
    n = (n * 2.0) - 1.0;

    if (length(n) < 0.9999) {
     clip(-1);
    }

    float3 p = pos.Sample(Sampler, uv).xyz;
    float4 c = albedo.Sample(Sampler, uv);

    float3 dir_light = normalize(float3(50.0, 10.0, 0.0));

    float sun_zenith = dot(float3(0.0, 1.0, 0.0), dir_light);

    float4 ir = irradiance.Sample(Sampler, GetIrradianceUVFromRMuS(6360.0, sun_zenith));

    float3 reflectance = (c.xyz / 3.14159) * (1 + dot(n.xyz, float3(0.0, 1.0, 0.0))) / 2;

    float4 reflect = ir * float4(reflectance.xyz, 1.0);

    //float angle = clamp(dot(dir_light, n.xyz), 0.0, 1.0);



    return float4(pow(reflect.xyz, float3(1.0 / 2.2, 1.0 / 2.2, 1.0 / 2.2)), 1.0) ;
}