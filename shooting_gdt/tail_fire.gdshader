shader_type spatial;

uniform vec4 fire_color: hint_color = vec4(0.0, 0.0, 1.0, 1.0);

varying vec3 nom;

const float PI2 = 6.28318530718;

void vertex() {
	nom = NORMAL;
}

void fragment() {
	ALBEDO = vec3(0.0);
	EMISSION = fire_color.rgb;
	// EMISSION = VERTEX;
	// ALPHA = VERTEX.z;
	// ALPHA = smoothstep(0.0, 1.0, 0.5 - nom.z) * float(int(TIME * 1000.0) % 2);
	// ALPHA = smoothstep(0.0, 1.0, 0.5 - nom.z) * sin(mod(TIME * 100.0, PI2));
	float speed = 0.1;
	ALPHA = smoothstep(0.0, 1.0, 0.5 - nom.z) * sin(mod(TIME, speed) / speed * PI2);
}