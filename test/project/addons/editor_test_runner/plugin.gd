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
		var tests = Tests.new()
		add_child(tests)
		tests.run()
	else:
		print("Opening editor normally for the test project. To run tests, pass `--run-editor-tests` to the executable.")
