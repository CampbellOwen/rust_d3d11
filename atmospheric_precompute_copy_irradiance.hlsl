
StructuredBuffer<float3> Buf : register(t0);
RWTexture2D<float4> Tex: register(u0);

[numthreads(32, 1, 1)]
void main (uint3 DTid: SV_DispatchThreadId) {
    uint index = (DTid.y * 64 ) + DTid.x;
    Tex[DTid.xy] = float4(Buf[index], 1.0);
}