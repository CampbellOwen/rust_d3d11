

struct Vout 
{
    float4 position : SV_POSITION;
    float3 ws_position: POSITIONT;
    float3 normal: NORMAL;
    float2 uv: TEXCOORD;
};

struct Vin {
    float3 position: POSITION;
    float3 normal: NORMAL;
    float2 uv : TEXCOORD;
    uint vertexId : SV_VertexID;
};

cbuffer FrameConstants : register(b0) {
    float4x4 WorldView;
}

cbuffer ModelConstants : register(b2) {
    float4x4 ModelWorld;
}


Vout vertex(Vin input) {
    Vout output;

    float4 pos = mul(ModelWorld, float4(input.position, 1.0));

    output.position = mul(WorldView, pos);
    output.ws_position = pos;
    output.normal = normalize(mul(float4(input.normal, 0.0), WorldView).xyz);
    output.uv = input.uv;

    return output;
}


struct Pout {
    float4 position: SV_Target0;
    float4 albedo: SV_Target1;
    float4 normal: SV_Target2;
};

Texture2D albedoTex : register(t0);
SamplerState Sampler;



Pout pixel(Vout input) {

    Pout output;

    float2 uv = input.uv;
    uv.y = 1.0 - uv.y;

    output.position = float4((input.ws_position * 0.5) + 0.5, 1.0);
    output.albedo = float4(albedoTex.Sample(Sampler, uv).xyz, 1.0);
    output.normal = float4((input.normal * 0.5) + 0.5, 1.0);

    return output;
}