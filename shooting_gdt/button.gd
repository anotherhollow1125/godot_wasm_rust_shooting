extends TouchScreenButton

var stick_vec: Vector2
var touched := false

# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
#func _process(delta):
#	pass

var sens = 0.2

func _process(delta):
	if touched:
		if stick_vec.x > sens:
			Input.action_release("ui_left");
			Input.action_press("ui_right");
		elif stick_vec.x < -sens:
			Input.action_release("ui_right");
			Input.action_press("ui_left");
		else:
			Input.action_release("ui_left");
			Input.action_release("ui_right");

		if stick_vec.y > sens:
			Input.action_release("ui_up");
			Input.action_press("ui_down");
		elif stick_vec.y < -sens:
			Input.action_release("ui_down");
			Input.action_press("ui_up");
		else:
			Input.action_release("ui_up");
			Input.action_release("ui_down");
			

	if !touched:
		stick_vec = Vector2(0.0, 0.0);

	var n_vec = stick_vec.normalized();
	
	if touched:
		$Stick/ball.position = 200.0 * n_vec;
	else:
		$Stick/ball.position = 200.0 * calc_other_input_vec();

func calc_other_input_vec() -> Vector2:
	var x = 0.0;
	x += 1.0 if Input.is_action_pressed("ui_right") else 0.0;
	x += -1.0 if Input.is_action_pressed("ui_left") else 0.0;
	var y = 0.0;
	y += 1.0 if Input.is_action_pressed("ui_down") else 0.0;
	y += -1.0 if Input.is_action_pressed("ui_up") else 0.0;
	return Vector2(x, y).normalized();

func _on_button_pressed():
	touched = true


func _on_button_released():
	Input.action_release("ui_right");
	Input.action_release("ui_left");
	Input.action_release("ui_up");
	Input.action_release("ui_down");
	touched = false
