#define INSCATTER_R_SIZE 32
#define INSCATTER_MU_SIZE 128
#define INSCATTER_MU_S_SIZE 32
#define INSCATTER_NU_SIZE 8

StructuredBuffer<float3> Rayleigh : register(t0);
StructuredBuffer<float3> Mie : register(t1);
RWTexture3D<float4> Tex: register(u0);

[numthreads(32, 1, 1)]
void main (uint3 DTid: SV_DispatchThreadId) {

    uint index = (DTid.z * INSCATTER_MU_S_SIZE * INSCATTER_MU_SIZE * INSCATTER_NU_SIZE) + (DTid.y * INSCATTER_MU_S_SIZE * INSCATTER_NU_SIZE + DTid.x);

    float3 ray = Rayleigh[index];
    float3 mie = Mie[index];

    Tex[DTid.xyz] = float4(ray, mie.r);
}