struct Vout 
{
    float4 position : SV_POSITION;
    float2 uv: TEXCOORD;
};


Vout main(uint vertexId : SV_VertexID) {
    Vout output;

    if (vertexId == 0) {
        output.position = float4(-1.0, -1.0, 0.5, 1.0);
        output.uv = float2(0.0, 1.0);
    }
    else if (vertexId == 1) {
        output.position = float4(-1.0,  1.0, 0.5, 1.0);
        output.uv = float2(0.0, 0.0);
    }
    else if (vertexId == 2) {
        output.position = float4( 1.0, -1.0, 0.5, 1.0);
        output.uv = float2(1.0, 1.0);
    }

    else if (vertexId == 3) {
        output.position = float4( 1.0, -1.0, 0.5, 1.0);
        output.uv = float2(1.0, 1.0);
    }
    else if (vertexId == 4) {
        output.position = float4(-1.0,  1.0, 0.5, 1.0);
        output.uv = float2(0.0, 0.0);
    }
    else if (vertexId == 5) {
        output.position = float4( 1.0,  1.0, 0.5, 1.0);
        output.uv = float2(1.0, 0.0);
    }
    else  {
        output.position = float4(0.0, 0.0, 0.0, 0.0);
        output.uv = float2(0.0, 1.0);
    }

    return output;
}