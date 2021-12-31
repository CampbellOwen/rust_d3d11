
Texture2D position : register(t0);
Texture2D albedo : register(t1);
Texture2D normal : register(t2);

SamplerState Sampler;

float4 main(float4 position : SV_POSITION, float2 uv: TEXCOORD) : SV_TARGET {

    return albedo.Sample(Sampler, uv);

}