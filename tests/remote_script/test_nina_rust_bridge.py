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


class FakeArrangementClip:
    def __init__(self, start_time, length):
        self.start_time = start_time
        self.length = length
        self.name = ""
        self.is_midi_clip = True
        self.notes = ()

    def set_notes(self, notes):
        self.notes = notes


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


if __name__ == "__main__":
    unittest.main()
