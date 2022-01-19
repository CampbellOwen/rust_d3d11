
Texture2D pos : register(t0);
Texture2D albedo : register(t1);
Texture2D normal : register(t2);

SamplerState Sampler;

float4 main(float4 position : SV_POSITION, float2 uv: TEXCOORD) : SV_TARGET {


    float4 n = normal.Sample(Sampler, uv);
    n = (n * 2.0) - 1.0;

    if (length(n) < 0.9999) {
     clip(-1);
    }

    float3 p = pos.Sample(Sampler, uv).xyz;
    float4 c = albedo.Sample(Sampler, uv);

    float3 dir_light = normalize(float3(0.0, 50.0, -10.0));


    float angle = clamp(dot(dir_light, n.xyz), 0.0, 1.0);


    return (c * angle);
    //return c;

}