class_name Tests
extends Node

var gdn

func run():
	var version = Engine.get_version_info()
	print(" -- Running on Godot ", version["string"])
	print(" -- Rust GDNative test suite:")
	_timeout()

	gdn = GDNative.new()
	var status = false;

	gdn.library = load("res://gdnative.gdnlib")

	if gdn.initialize():
		status = gdn.call_native("standard_varcall", "run_tests", [])

		status = status && _test_argument_passing_sanity()
		status = status && _test_generic_class()
		status = status && _test_optional_args()
		status = status && yield(_test_async_resume(), "completed")

		# Godot needs another frame to dispose the executor driver node. Otherwise the process
		# aborts due to `_process` being called after `terminate` (`get_api` fail, not UB).
		yield(get_tree().create_timer(0.1), "timeout")

		gdn.terminate()
	else:
		print(" -- Could not load the GDNative library.")

	print()
	if status:
		print(" All tests PASSED.")
	else:
		print(" Tests FAILED.")
		OS.exit_code = 1

	print(" -- exiting.")
	get_tree().quit()

func _timeout():
	yield(get_tree().create_timer(10.0), "timeout")
	print(" -- Test run is taking too long.")
	OS.exit_code = 1

	print(" -- exiting.")
	get_tree().quit()

func _test_argument_passing_sanity():
	print(" -- test_argument_passing_sanity")

	var script = NativeScript.new()
	script.set_library(gdn.library)
	script.set_class_name("Foo")
	var foo = script.new()
	
	var status = true

	status = status && _assert_choose("foo", foo, "choose", "foo", true, "bar")
	status = status && _assert_choose("night", foo, "choose", "day", false, "night")
	status = status && _assert_choose(42, foo, "choose_variant", 42, "int", 54.0)
	status = status && _assert_choose(9.0, foo, "choose_variant", 6, "float", 9.0)

	if status:
		assert("foo" == foo.choose("foo", true, "bar"))
		assert("night" == foo.choose("day", false, "night"))
		assert(42 == foo.choose_variant(42, "int", 54.0))
		assert(9.0 == foo.choose_variant(6, "float", 9.0))

	if !status:
		printerr("   !! test_argument_passing_sanity failed")

	return status

func _assert_choose(expected, foo, fun, a, which, b):
	var got_value = foo.call(fun, a, which, b)
	if got_value == expected:
		return true
	printerr("   !! expected ", expected, ", got ", got_value)
	return false

func _test_optional_args():
	print(" -- _test_optional_args")
	print("   -- expected error messages for edge cases:")
	print("     -- missing non-optional parameter `b` (#1)")
	print("     -- 1 excessive argument is given: [I64(6)]")
	print("   -- the test is successful when and only when these errors are shown")

	var script = NativeScript.new()
	script.set_library(gdn.library)
	script.set_class_name("OptionalArgs")
	var opt_args = script.new()

	var status = true

	status = status && _assert_opt_args(null, opt_args, [1])
	status = status && _assert_opt_args(2, opt_args, [1, 1])
	status = status && _assert_opt_args(6, opt_args, [1, 3, 2])
	status = status && _assert_opt_args(13, opt_args, [5, 1, 3, 4])
	status = status && _assert_opt_args(42, opt_args, [4, 1, 20, 4, 13])
	status = status && _assert_opt_args(null, opt_args, [1, 2, 3, 4, 5, 6])

	if !status:
		printerr("   !! _test_optional_args failed")

	return status

func _assert_opt_args(expected, opt_args, args):
	var got_value = opt_args.callv("opt_sum", args);
	if got_value == expected:
		return true
	printerr("   !! expected ", expected, ", got ", got_value)
	return false


func _test_async_resume():
	print(" -- _test_async_resume")

	var driver_script = NativeScript.new()
	driver_script.set_library(gdn.library)
	driver_script.set_class_name("AsyncExecutorDriver")
	var driver = driver_script.new()
	add_child(driver)

	var script = NativeScript.new()
	script.set_library(gdn.library)
	script.set_class_name("AsyncMethods")
	var resume = script.new()

	var status = true

	# Force this to return a FunctionState for convenience
	yield(get_tree().create_timer(0.1), "timeout")

	var fn_state = resume.resume_add(1, self, "_get_async_number")
	if !fn_state:
		printerr("   !! _test_async_resume failed")
		remove_child(driver)
		driver.queue_free()
		return false

	yield(fn_state, "resumable")
	status = status && fn_state.is_valid()
	
	fn_state = fn_state.resume(2)
	if !fn_state:
		printerr("   !! _test_async_resume failed")
		remove_child(driver)
		driver.queue_free()
		return false

	var result = yield(fn_state, "completed")
	
	status = status && (result == 42)

	if !status:
		printerr("   !! _test_async_resume failed")

	remove_child(driver)
	driver.queue_free()

	return status

func _get_async_number():
	yield(get_tree().create_timer(0.1), "timeout")
	return 39

func _test_generic_class():
	print(" -- _test_generic_class")

	var int_script = NativeScript.new()
	int_script.set_library(gdn.library)
	int_script.set_class_name("IntOps")

	var str_script = NativeScript.new()
	str_script.set_library(gdn.library)
	str_script.set_class_name("StrOps")
	
	var int_ops = int_script.new()
	
	var str_ops = str_script.new()

	var status = true

	status = status && int_ops.add(1, 2) == 3
	status = status && int_ops.sub(3, 2) == 1
	status = status && str_ops.add("foo", "bar") == "foobar"

	if !status:
		printerr("   !! _test_generic_class failed")

	return status
