extends Panel

func _ready():
	pass # Replace with function body.

# There is a signal set up on the Add button which calls
#   this method. We pass the call into Parent.spawn_one
#   (This calls Rust)
func _on_Add_pressed():
	var spawner_node = get_node("/root/Main/Parent")
	spawner_node.spawn_one("example string")

# There is a signal set up on the Remove button which calls
#   this method. We pass the call into Parent.remove_one
#   (This calls Rust)
func _on_Remove_pressed():
	var spawner_node = get_node("/root/Main/Parent")
	spawner_node.remove_one("Another example string")

# This function is called from Rust. All we need there is this 
#   node and the name "set_num_children"
func set_num_children(children_count):
	# This is called from rust
	if children_count == 0 :
		$Label.text = "What did you go and kill all the children for!?"
	elif children_count == 1:
		$Label.text = "We have one child"
	else :
		$Label.text = "%d children have been created so far" % children_count
		
	
	