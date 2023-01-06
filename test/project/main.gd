extends Node

func _ready():
	var tests = Tests.new()
	add_child(tests)
	tests.run()