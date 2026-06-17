extends Node3D

@onready var m_timer: MTimer = $MTimer

func _ready() -> void:
	m_timer.timeout.connect(func(): print("AHHHH"))
	m_timer.start(4.0)
	print(m_timer.time_left)
