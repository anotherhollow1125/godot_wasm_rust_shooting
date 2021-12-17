shader_type spatial;

const float PI2 = 6.28318530718;

// https://nn-hokuson.hatenablog.com/entry/2017/01/27/195659

float random(vec2 uv) {
	return fract(sin(dot(uv - mod(uv, 0.001), vec2(12.9898,78.233))) * 43758.5453);
	return fract(sin(dot(uv - mod(uv, 0.001), vec2(12.9898,78.233))) * 43758.5453);
}

float mod_time(float t, float m) {
	return mod(t, m) / m;
}

void fragment() {
	float b = 0.97;
	float c = random(UV);
	float d = random(UV.yx);
	
	float time_sin = sin(mod(mod_time(TIME, 5.0) + random(UV + vec2(0.5)), 1.0) * PI2);
	c = step(b, c) * step(b, d) * 0.5 * time_sin;
	ALBEDO = vec3(c);
}