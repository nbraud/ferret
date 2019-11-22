#version 450

layout(set = 0, binding = 0) uniform UniformBufferObject {
	mat4 view;
	mat4 proj;
} ubo;

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec3 in_texture_coord;
layout(location = 2) in vec3 in_lightmap_coord;

layout(location = 0) out vec3 frag_texture_coord;
layout(location = 1) out vec3 frag_lightmap_coord;

out gl_PerVertex {
	vec4 gl_Position;
};

void main()
{
	frag_texture_coord = in_texture_coord;
	frag_lightmap_coord = in_lightmap_coord;
	gl_Position = ubo.proj * ubo.view * vec4(in_position, 1);
}
