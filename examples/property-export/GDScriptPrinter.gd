extends Node

func _ready():
	var rust = get_node("../PropertyExport")

	print("\n-----------------------------------------------------------------")
	print("Print from GDScript:")
	print("  PoolArray<GodotString>:");
	for name in rust.name_vec:
		print("  * %s" % name)

	print("\n  Dictionary (string -> color):")
	for string in rust.color_map:
		var color = rust.color_map[string]
		print("  * %s -> #%s" % [string, color.to_html(false)]);

	print("\n  PoolArray<i32>:")
	for id in rust.id_set:
		print("  * %s" % id)

	# The program has printed the contents and fulfilled its purpose, quit
	get_tree().quit()