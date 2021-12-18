extends Control

signal game_start

# Declare member variables here. Examples:
# var a = 2
# var b = "text"
var start_flag = false

# Called when the node enters the scene tree for the first time.
func _ready():
	$GameOverGroup.visible = false


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	if !start_flag:
		start_game_watcher()

func start_game_watcher():
	if Input.is_action_pressed("shoot"):
		start_flag = true
		$TitleGroup.visible = false
		emit_signal("game_start")

func _on_stage_set_remain(remain):
	$Panel/RemainLabel.text = str(remain - 1 if remain > 1 else 0)


func _on_stage_set_score(score):
	$Panel/SocreValLabel.text = str(score)


func _on_ResetButton_pressed():
	get_tree().reload_current_scene()


func _on_stage_game_over():
	$GameOverGroup.visible = true
	$GameOverGroup/ScoreLabel.text = "SCORE : " + $Panel/SocreValLabel.text
