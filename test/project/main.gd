extends Node

var gdn

func _ready():
    print(" -- Rust gdnative test suite:")
    gdn = GDNative.new()
    var status = false;

    gdn.library = load("res://gdnative.gdnlib")

    if gdn.initialize():
        status = gdn.call_native("standard_varcall", "run_tests", [])

        status = status && _test_argument_passing_sanity()

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

func _test_argument_passing_sanity():
    print(" -- test_argument_passing_sanity")

    var script = NativeScript.new()
    script.set_library(gdn.library)
    script.set_class_name("Foo")
    var foo = Reference.new()
    foo.set_script(script)
    
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