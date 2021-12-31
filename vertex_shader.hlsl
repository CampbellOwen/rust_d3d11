struct Vout 
{
    float4 position : SV_POSITION;
    float2 uv: TEXCOORD;
};


Vout main(float3 position: POSITION, float2 uv : TEXCOORD) {
    Vout output;
    output.position = float4(position, 1.0);
    output.uv = uv;

    return output;
}