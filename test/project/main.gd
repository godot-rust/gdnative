extends Node

func _ready():
    print(" -- Rust gdnative test suite:")
    var gdn = GDNative.new()
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
