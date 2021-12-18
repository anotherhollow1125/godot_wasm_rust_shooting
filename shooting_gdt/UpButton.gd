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
	var now_flag = Input.is_action_pressed("ui_up")
	if now_flag != flag:
		if now_flag:
			set_texture(pressed)
		else:
			set_texture(original_normal)
		flag = now_flag
