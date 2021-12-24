extends Sprite

func _ready():
	pass

func _input(event):
	if !(event is InputEventScreenTouch) and \
		!(event is InputEventScreenDrag):
		return
	
	var p = to_local(event.position);
	if !get_rect().has_point(p):
		# $"../".stick_vec = Vector2(0.0, 0.0);
		return

	var threshold = abs(p.x) > 50 or abs(p.y) > 50;
	var leftright_vec = p.x / 500.0 if threshold else 0;
	var updown_vec = p.y / 500.0 if threshold else 0;
	var new_vec = Vector2(leftright_vec, updown_vec);
	# print(new_vec)
	$"../".stick_vec = new_vec;
