

struct Vout 
{
    float4 position : SV_POSITION;
    float3 ws_position: POSITIONT;
    float4 colour: COLOR;
    float3 normal: NORMAL;
};


Vout vertex(float3 position: POSITION, float4 colour : COLOR) {
    Vout output;
    output.position = float4(position, 1.0);
    output.ws_position = position;
    output.colour = colour;
    output.normal = float3(0.0, 0.0, 1.0);

    return output;
}

struct Pout {
    float4 position: SV_Target0;
    float4 albedo: SV_Target1;
    float4 normal: SV_Target2;
};


Pout pixel(float4 position : SV_POSITION, float3 ws_position : POSITIONT, float4 colour: COLOR, float3 normal: NORMAL) {

    Pout output;

    output.position = float4((ws_position * 0.5) + 0.5, 1.0);
    output.albedo = colour;
    output.normal = float4((normal * 0.5) + 0.5, 1.0);

    return output;
}