extends TouchScreenButton


# Declare member variables here. Examples:
# var a = 2
# var b = "text"
var original_normal = normal
var flag = false

# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	var now_flag = Input.is_action_pressed("shoot")
	if now_flag != flag:
		if now_flag:
			set_texture(pressed)
			$ShotLabel.add_color_override("font_color", Color(33.0 / 256.0, 34.0  / 256.0, 64.0  / 256.0))
		else:
			set_texture(original_normal)
			$ShotLabel.add_color_override("font_color", Color(1.0, 1.0, 1.0))
		flag = now_flag
