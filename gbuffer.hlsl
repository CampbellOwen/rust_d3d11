

struct Vout 
{
    float4 position : SV_POSITION;
    float3 ws_position: POSITIONT;
    float4 colour: COLOR;
    float3 normal: NORMAL;
    float2 uv: TEXCOORD;
};

struct Vin {
    float3 position: POSITION;
    float3 normal: NORMAL;
    float2 uv : TEXCOORD;
    uint vertexId : SV_VertexID;
};


Vout vertex(Vin input) {
    Vout output;
    output.position = float4(input.position, 1.0);
    output.ws_position = input.position;
    output.colour = input.vertexId == 0 ? float4(1.0, 0.0, 0.0, 1.0) : input.vertexId == 1 ? float4(0.0, 1.0, 0.0, 1.0) : float4(0.0, 0.0, 1.0, 1.0);
    output.normal = input.normal;
    output.uv = input.uv;

    return output;
}


struct Pout {
    float4 position: SV_Target0;
    float4 albedo: SV_Target1;
    float4 normal: SV_Target2;
};


Pout pixel(Vout input) {

    Pout output;

    output.position = float4((input.ws_position * 0.5) + 0.5, 1.0);
    output.albedo = input.colour;
    output.normal = float4((input.normal * 0.5) + 0.5, 1.0);

    return output;
}