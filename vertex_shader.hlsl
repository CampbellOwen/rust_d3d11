struct Vout 
{
    float4 position : SV_POSITION;
    float4 colour: COLOR;
};


Vout main(float3 position: POSITION, float4 colour : COLOR) {
    Vout output;
    output.position = float4(position, 1.0);
    output.colour = colour;

    return output;
}