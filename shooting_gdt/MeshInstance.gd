extends MeshInstance


# Declare member variables here. Examples:
# var a = 2
# var b = "text"

var speed = 0.1;
const original_speed = 0.1;

# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	translate(Vector3(0.0, 0.0, delta * speed));
	if translation.z > 150.0:
		translation.z = -150.0;

func _on_stage_speed_up(val):
	speed = original_speed * val;
	# print(speed)
