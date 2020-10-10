tool
extends EditorPlugin

var gdn

func _enter_tree():
	var run_tests = false
	for arg in OS.get_cmdline_args():
		if arg == "--run-editor-tests":
			run_tests = true
			break
	if run_tests:
		_run_tests()
	else:
		print("Opening editor normally for the test project. To run tests, pass `--run-editor-tests` to the executable.")

func _run_tests():
	print(" -- Rust gdnative test suite:")
	gdn = GDNative.new()
	var status = false;

	gdn.library = load("res://gdnative.gdnlib")

	if gdn.initialize():
		status = gdn.call_native("standard_varcall", "run_tests", [])

		gdn.terminate()
	else:
		print(" -- Could not load the gdnative library.")

	if status:
		print(" -- Test run completed successfully.")
	else:
		print(" -- Test run completed with errors.")
		OS.exit_code = 1

	print(" -- exiting.")
	get_tree().quit()

