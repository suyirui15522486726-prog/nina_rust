import importlib.util
import pathlib
import sys
import threading
import types
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SCRIPT_PATH = ROOT / "remote_scripts" / "NinaRustBridge" / "__init__.py"


class FakeTrack:
    def __init__(self, name, has_midi_input=True, has_audio_input=False):
        self.name = name
        self.has_midi_input = has_midi_input
        self.has_audio_input = has_audio_input
        self.devices = []
        self.clip_slots = []
        self.arrangement_clips = []

    def create_midi_clip(self, position, length):
        clip = FakeArrangementClip(position, length)
        self.arrangement_clips.append(clip)
        return clip


class FakeParameter:
    def __init__(
        self,
        name,
        value=0.0,
        min_value=0.0,
        max_value=1.0,
        display_value=None,
        is_enabled=True,
        is_quantized=False,
    ):
        self.name = name
        self.value = value
        self.min = min_value
        self.max = max_value
        self._display_value = display_value or str(value)
        self.is_enabled = is_enabled
        self.is_quantized = is_quantized

    def str_for_value(self, _value):
        return self._display_value


class FakeDevice:
    def __init__(
        self,
        name,
        class_name,
        parameters=None,
        chains=None,
        device_type=None,
        drum_pads=None,
    ):
        self.name = name
        self.class_name = class_name
        self.parameters = list(parameters or [])
        self.chains = list(chains or [])
        self.type = device_type
        self.drum_pads = list(drum_pads or [])


class FakeChain:
    def __init__(self, name, devices=None, out_note=None):
        self.name = name
        self.devices = list(devices or [])
        self.out_note = out_note


class FakeDrumPad:
    def __init__(self, name, note, chains=None):
        self.name = name
        self.note = note
        self.chains = list(chains or [])


class FakeArrangementClip:
    def __init__(self, start_time, length):
        self.start_time = start_time
        self.length = length
        self.name = ""
        self.is_midi_clip = True
        self.notes = ()

    def set_notes(self, notes):
        self.notes = notes

    def get_notes(self, _start_time, _pitch, _time_span, _pitch_span):
        return self.notes


class FakeSong:
    def __init__(self):
        self.tempo = 120.0
        self.signature_numerator = 4
        self.signature_denominator = 4
        self.is_playing = False
        self.tracks = [
            FakeTrack("Drums"),
            FakeTrack("Bass"),
            FakeTrack("Vocal", has_midi_input=False, has_audio_input=True),
        ]
        self.scenes = [object()]
        self.return_tracks = []
        self.master_track = object()

    def create_midi_track(self, index):
        if index < 0 or index > len(self.tracks):
            raise IndexError("Track index out of range")
        self.tracks.insert(index, FakeTrack("New MIDI Track"))

    def start_playing(self):
        self.is_playing = True

    def stop_playing(self):
        self.is_playing = False


class DeferredTransportSong(FakeSong):
    def __init__(self, initial_is_playing):
        super().__init__()
        self.is_playing = initial_is_playing
        self.start_called = False
        self.stop_called = False

    def start_playing(self):
        self.start_called = True

    def stop_playing(self):
        self.stop_called = True


class FakeControlSurface:
    def __init__(self, c_instance=None):
        self.c_instance = c_instance
        self.messages = []
        self._fake_song = FakeSong()
        self._fake_application = FakeApplication()

    def song(self):
        return self._fake_song

    def application(self):
        return getattr(self, "_fake_application", FakeApplication())

    def log_message(self, message):
        if not hasattr(self, "messages"):
            self.messages = []
        self.messages.append(("log", message))

    def show_message(self, message):
        if not hasattr(self, "messages"):
            self.messages = []
        self.messages.append(("show", message))

    def schedule_message(self, _delay, callback):
        callback()

    def disconnect(self):
        self.messages.append(("disconnect", "called"))


class FakeBrowserItem:
    def __init__(self, name, is_folder=False, is_loadable=False, uri=None, children=None):
        self.name = name
        self.is_folder = is_folder
        self.is_loadable = is_loadable
        self.uri = uri
        self._children = list(children or [])

    @property
    def iter_children(self):
        return tuple(self._children)


class FakeBrowser:
    def __init__(self):
        self.sounds = (
            FakeBrowserItem("Cold Synths", is_folder=True, uri="browser://sounds/cold"),
            FakeBrowserItem("Glass Pad", is_loadable=True, uri="browser://sounds/glass"),
        )
        self.instruments = (FakeBrowserItem("Wavetable", is_loadable=True),)


class FakeApplication:
    def __init__(self, browser=None):
        self.browser = browser or FakeBrowser()


def load_bridge_module():
    framework = types.ModuleType("_Framework")
    control_surface = types.ModuleType("_Framework.ControlSurface")
    control_surface.ControlSurface = FakeControlSurface
    sys.modules["_Framework"] = framework
    sys.modules["_Framework.ControlSurface"] = control_surface

    spec = importlib.util.spec_from_file_location("nina_rust_bridge_test_module", SCRIPT_PATH)
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


class NinaRustBridgeTests(unittest.TestCase):
    def test_bridge_constants_use_localhost_and_expected_name(self):
        module = load_bridge_module()

        self.assertEqual(module.BRIDGE_NAME, "NinaRustBridge")
        self.assertEqual(module.HOST, "127.0.0.1")
        self.assertEqual(module.PORT, 9878)

    def test_health_check_returns_bridge_identity(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident

        response = bridge._process_command({"type": "health_check", "params": {"ping": "pong"}})

        self.assertEqual(response["status"], "success")
        self.assertEqual(response["result"]["ok"], True)
        self.assertEqual(response["result"]["name"], "NinaRustBridge")
        self.assertEqual(response["result"]["echo"], {"ping": "pong"})

    def test_unknown_command_returns_error_response(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident

        response = bridge._process_command({"type": "does_not_exist", "params": {}})

        self.assertEqual(response["status"], "error")
        self.assertIn("Unknown NinaRustBridge command", response["message"])

    def test_snapshot_live_set_summarizes_fake_song(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident
        bridge._song = FakeSong()

        response = bridge._process_command({"type": "snapshot_live_set", "params": {}})

        self.assertEqual(response["status"], "success")
        self.assertEqual(response["result"]["tempo"], 120.0)
        self.assertEqual(response["result"]["track_count"], 3)
        self.assertEqual(response["result"]["tracks"][0]["name"], "Drums")

    def test_transport_commands_return_requested_state_even_if_live_updates_later(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident

        bridge._song = DeferredTransportSong(initial_is_playing=False)
        play = bridge._process_command({"type": "start_playback", "params": {}})
        self.assertEqual(play["status"], "success")
        self.assertEqual(play["result"]["is_playing"], True)
        self.assertEqual(bridge._song.start_called, True)

        bridge._song = DeferredTransportSong(initial_is_playing=True)
        stop = bridge._process_command({"type": "stop_playback", "params": {}})
        self.assertEqual(stop["status"], "success")
        self.assertEqual(stop["result"]["is_playing"], False)
        self.assertEqual(bridge._song.stop_called, True)

    def test_create_midi_clip_range_creates_arrangement_clip_on_midi_track(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident
        bridge._song = FakeSong()

        response = bridge._process_command(
            {
                "type": "create_midi_clip_range",
                "params": {
                    "track_index": 0,
                    "start_bar": 1,
                    "end_bar": 5,
                    "name": "Contract Clip",
                },
            }
        )

        self.assertEqual(response["status"], "success")
        self.assertEqual(response["result"]["track_name"], "Drums")
        self.assertEqual(response["result"]["start_time"], 0.0)
        self.assertEqual(response["result"]["length"], 16.0)
        self.assertEqual(response["result"]["clip_name"], "Contract Clip")
        self.assertEqual(len(bridge._song.tracks[0].arrangement_clips), 1)

    def test_create_midi_clip_range_rejects_audio_track(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident
        bridge._song = FakeSong()

        response = bridge._process_command(
            {
                "type": "create_midi_clip_range",
                "params": {"track_index": 2, "start_bar": 1, "end_bar": 3},
            }
        )

        self.assertEqual(response["status"], "error")
        self.assertIn("not a MIDI track", response["message"])

    def test_write_midi_clip_sets_notes_on_created_arrangement_clip(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident
        bridge._song = FakeSong()

        response = bridge._process_command(
            {
                "type": "write_midi_clip",
                "params": {
                    "track_index": 1,
                    "start_bar": 1,
                    "end_bar": 3,
                    "name": "Contract Phrase",
                    "notes": [
                        {"pitch": 48, "start": 0.0, "duration": 0.5, "velocity": 100},
                        {"pitch": 55, "start": 0.5, "duration": 0.5, "velocity": 90},
                    ],
                },
            }
        )

        self.assertEqual(response["status"], "success")
        self.assertEqual(response["result"]["note_count"], 2)
        clip = bridge._song.tracks[1].arrangement_clips[0]
        self.assertEqual(clip.name, "Contract Phrase")
        self.assertEqual(clip.notes[0], (48, 0.0, 0.5, 100, False))
        self.assertEqual(clip.notes[1], (55, 0.5, 0.5, 90, False))

    def test_export_midi_track_returns_all_arrangement_clip_notes_on_track_timeline(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident
        bridge._song = FakeSong()

        intro = FakeArrangementClip(0.0, 4.0)
        intro.name = "Intro"
        intro.set_notes(
            (
                (36, 0.0, 0.5, 110, False),
                (38, 1.0, 0.5, 96, False),
            )
        )
        drop = FakeArrangementClip(8.0, 2.0)
        drop.name = "Drop"
        drop.set_notes(((42, 0.0, 0.25, 76, False),))
        bridge._song.tracks[1].arrangement_clips = [intro, drop]

        response = bridge._process_command(
            {"type": "export_midi_track", "params": {"track_index": 1}}
        )

        self.assertEqual(response["status"], "success")
        self.assertEqual(response["result"]["track"]["name"], "Bass")
        self.assertEqual(response["result"]["clip_count"], 2)
        self.assertEqual(response["result"]["note_count"], 3)
        self.assertEqual([note["start"] for note in response["result"]["notes"]], [0.0, 1.0, 8.0])
        self.assertEqual(response["result"]["clips"][1]["name"], "Drop")
        self.assertEqual(response["result"]["end_beat"], 10.0)

    def test_create_midi_track_inserts_named_track(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident
        bridge._song = FakeSong()

        response = bridge._process_command(
            {
                "type": "create_midi_track",
                "params": {"index": 1, "name": "LLM Synth"},
            }
        )

        self.assertEqual(response["status"], "success")
        self.assertEqual(response["result"]["index"], 1)
        self.assertEqual(response["result"]["name"], "LLM Synth")
        self.assertEqual(response["result"]["track_count"], 4)
        self.assertEqual(bridge._song.tracks[1].name, "LLM Synth")

    def test_browser_scan_root_returns_first_level_items(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident
        bridge._song = FakeSong()
        bridge._fake_application = FakeApplication()

        response = bridge._process_command(
            {"type": "browser_scan_root", "params": {"root": "sounds", "limit": 1}}
        )

        self.assertEqual(response["status"], "success")
        self.assertEqual(response["result"]["root"], "sounds")
        self.assertEqual(response["result"]["count"], 1)
        self.assertEqual(response["result"]["truncated"], True)
        self.assertEqual(response["result"]["items"][0]["name"], "Cold Synths")
        self.assertEqual(response["result"]["items"][0]["path"], "sounds/Cold Synths")

    def test_device_scan_track_returns_device_chain_overview(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident
        bridge._song = FakeSong()
        bridge._song.tracks[1].devices = [
            FakeDevice(
                "Scale",
                "MidiScale",
                parameters=[FakeParameter("Device On", 1.0, display_value="On")],
            ),
            FakeDevice(
                "12 String Chords Guitar",
                "InstrumentGroupDevice",
                parameters=[
                    FakeParameter("Device On", 1.0, display_value="On"),
                    FakeParameter("Mass", 0.88, display_value="88 %"),
                ],
                chains=[
                    FakeChain(
                        "Hammer",
                        devices=[
                            FakeDevice(
                                "Hammer Body",
                                "Collision",
                                parameters=[FakeParameter("Damping", 0.7, display_value="70 %")],
                            )
                        ],
                    )
                ],
            ),
            FakeDevice(
                "Reverb",
                "Reverb",
                parameters=[FakeParameter("Dry/Wet", 0.32, display_value="32 %")],
            ),
        ]

        response = bridge._process_command(
            {"type": "device_scan_track", "params": {"track_index": 1}}
        )

        self.assertEqual(response["status"], "success")
        self.assertEqual(response["result"]["track"]["name"], "Bass")
        self.assertEqual(response["result"]["device_count"], 3)
        self.assertEqual(response["result"]["devices"][0]["role"], "midi_effect")
        self.assertEqual(response["result"]["devices"][1]["role"], "instrument_rack")
        self.assertEqual(response["result"]["devices"][1]["parameter_count"], 2)
        self.assertEqual(response["result"]["devices"][1]["chain_count"], 1)
        self.assertEqual(response["result"]["devices"][1]["parameters"], [])
        self.assertEqual(response["result"]["devices"][1]["chains"][0]["name"], "Hammer")
        self.assertEqual(
            response["result"]["devices"][1]["chains"][0]["devices"][0]["class_name"],
            "Collision",
        )
        self.assertEqual(
            response["result"]["devices"][1]["chains"][0]["devices"][0]["parameters"], []
        )
        self.assertEqual(response["result"]["devices"][2]["role"], "audio_effect")

    def test_device_scan_track_can_include_parameter_values(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident
        bridge._song = FakeSong()
        bridge._song.tracks[1].devices = [
            FakeDevice(
                "12 String Chords Guitar",
                "InstrumentGroupDevice",
                parameters=[
                    FakeParameter("Device On", 1.0, display_value="On"),
                    FakeParameter("Mass", 0.88, display_value="88 %"),
                ],
            )
        ]

        response = bridge._process_command(
            {
                "type": "device_scan_track",
                "params": {"track_index": 1, "include_parameters": True},
            }
        )

        self.assertEqual(response["status"], "success")
        self.assertEqual(response["result"]["devices"][0]["parameter_count"], 2)
        self.assertEqual(response["result"]["devices"][0]["parameters"][1]["name"], "Mass")
        self.assertEqual(
            response["result"]["devices"][0]["parameters"][1]["display_value"], "88 %"
        )

    def test_drum_scan_track_returns_used_pad_note_map(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident
        bridge._song = FakeSong()
        bridge._song.tracks[1].devices = [
            FakeDevice(
                "UKG Kit",
                "DrumGroupDevice",
                drum_pads=[
                    FakeDrumPad(
                        "Kick",
                        36,
                        chains=[
                            FakeChain(
                                "Kick",
                                devices=[FakeDevice("Kick Simpler", "OriginalSimpler")],
                                out_note=36,
                            )
                        ],
                    ),
                    FakeDrumPad(
                        "Snare",
                        38,
                        chains=[
                            FakeChain(
                                "Snare",
                                devices=[FakeDevice("Snare Simpler", "OriginalSimpler")],
                                out_note=38,
                            )
                        ],
                    ),
                    FakeDrumPad(
                        "Closed Hat",
                        42,
                        chains=[
                            FakeChain(
                                "Closed Hat",
                                devices=[FakeDevice("Hat Simpler", "OriginalSimpler")],
                                out_note=42,
                            )
                        ],
                    ),
                    FakeDrumPad("", 43, chains=[]),
                ],
            )
        ]

        response = bridge._process_command(
            {"type": "drum_scan_track", "params": {"track_index": 1}}
        )

        self.assertEqual(response["status"], "success")
        self.assertEqual(response["result"]["track"]["name"], "Bass")
        self.assertEqual(response["result"]["rack_count"], 1)
        rack = response["result"]["racks"][0]
        self.assertEqual(rack["name"], "UKG Kit")
        self.assertEqual(rack["used_pad_count"], 3)
        self.assertEqual([pad["role_guess"] for pad in rack["pads"]], ["kick", "snare", "closed_hat"])
        self.assertEqual([pad["note"] for pad in rack["pads"]], [36, 38, 42])
        self.assertEqual([pad["note_name"] for pad in rack["pads"]], ["C1", "D1", "F#1"])
        self.assertEqual(rack["pads"][0]["chains"][0]["devices"][0]["name"], "Kick Simpler")


if __name__ == "__main__":
    unittest.main()
