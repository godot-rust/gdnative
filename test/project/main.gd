extends Node

func _ready():
    print(" -- Rust gdnative test suite:")
    var gdn = GDNative.new()
    gdn.library = load("res://gdnative.gdnlib")
    if gdn.initialize():
        var status = gdn.call_native("standard_varcall", "run_tests", [])
        gdn.terminate()
        if status:
            print(" -- Test run completed successfully.")
        else:
            print(" -- Test run completed with errors.")
            OS.exit_code = 1
    else:
        print(" -- Could not load the gdnative library.")
    print(" -- exiting.")
    get_tree().quit()
