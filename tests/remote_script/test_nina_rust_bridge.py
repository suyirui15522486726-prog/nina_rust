# 本文件作用：验证 Ableton Remote Script 桥接命令的 Python 侧行为。

import importlib.util
import pathlib
import sys
import threading
import types
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SCRIPT_PATH = ROOT / "remote_scripts" / "NinaRustBridge" / "__init__.py"


# 类作用：封装 Fake Track 的状态和行为。
class FakeTrack:
    # 函数作用：执行 init 相关逻辑。
    def __init__(self, name, has_midi_input=True, has_audio_input=False):
        self.name = name
        self.has_midi_input = has_midi_input
        self.has_audio_input = has_audio_input
        self.devices = []
        self.clip_slots = []
        self.arrangement_clips = []

    # 函数作用：创建 midi clip。
    def create_midi_clip(self, position, length):
        clip = FakeArrangementClip(position, length)
        self.arrangement_clips.append(clip)
        return clip

    # 函数作用：创建 audio clip。
    def create_audio_clip(self, file_path, destination_time):
        clip = FakeArrangementClip(destination_time, 4.0)
        clip.name = pathlib.Path(file_path).name
        clip.file_path = file_path
        clip.is_midi_clip = False
        clip.is_audio_clip = True
        self.arrangement_clips.append(clip)
        return clip


# 类作用：封装 Fake Parameter 的状态和行为。
class FakeParameter:
    # 函数作用：执行 init 相关逻辑。
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

    # 函数作用：执行 str for value 相关逻辑。
    def str_for_value(self, _value):
        return self._display_value


# 类作用：封装 Fake Device 的状态和行为。
class FakeDevice:
    # 函数作用：执行 init 相关逻辑。
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


# 类作用：封装 Fake Chain 的状态和行为。
class FakeChain:
    # 函数作用：执行 init 相关逻辑。
    def __init__(self, name, devices=None, out_note=None):
        self.name = name
        self.devices = list(devices or [])
        self.out_note = out_note


# 类作用：封装 Fake Drum Pad 的状态和行为。
class FakeDrumPad:
    # 函数作用：执行 init 相关逻辑。
    def __init__(self, name, note, chains=None):
        self.name = name
        self.note = note
        self.chains = list(chains or [])


# 类作用：封装 Fake Arrangement Clip 的状态和行为。
class FakeArrangementClip:
    # 函数作用：执行 init 相关逻辑。
    def __init__(self, start_time, length):
        self.start_time = start_time
        self.length = length
        self.name = ""
        self.is_midi_clip = True
        self.is_audio_clip = False
        self.file_path = None
        self.notes = ()

    # 函数作用：执行 set notes 相关逻辑。
    def set_notes(self, notes):
        self.notes = notes

    # 函数作用：执行 get notes 相关逻辑。
    def get_notes(self, _start_time, _pitch, _time_span, _pitch_span):
        return self.notes


# 类作用：封装 Fake Song 的状态和行为。
class FakeSong:
    # 函数作用：执行 init 相关逻辑。
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

    # 函数作用：创建 midi track。
    def create_midi_track(self, index):
        if index < 0 or index > len(self.tracks):
            raise IndexError("Track index out of range")
        self.tracks.insert(index, FakeTrack("New MIDI Track"))

    # 函数作用：创建 audio track。
    def create_audio_track(self, index):
        if index < 0 or index > len(self.tracks):
            raise IndexError("Track index out of range")
        self.tracks.insert(
            index, FakeTrack("New Audio Track", has_midi_input=False, has_audio_input=True)
        )

    # 函数作用：执行 start playing 相关逻辑。
    def start_playing(self):
        self.is_playing = True

    # 函数作用：执行 stop playing 相关逻辑。
    def stop_playing(self):
        self.is_playing = False


# 类作用：封装 Deferred Transport Song 的状态和行为。
class DeferredTransportSong(FakeSong):
    # 函数作用：执行 init 相关逻辑。
    def __init__(self, initial_is_playing):
        super().__init__()
        self.is_playing = initial_is_playing
        self.start_called = False
        self.stop_called = False

    # 函数作用：执行 start playing 相关逻辑。
    def start_playing(self):
        self.start_called = True

    # 函数作用：执行 stop playing 相关逻辑。
    def stop_playing(self):
        self.stop_called = True


# 类作用：封装 Fake Control Surface 的状态和行为。
class FakeControlSurface:
    # 函数作用：执行 init 相关逻辑。
    def __init__(self, c_instance=None):
        self.c_instance = c_instance
        self.messages = []
        self._fake_song = FakeSong()
        self._fake_application = FakeApplication()

    # 函数作用：执行 song 相关逻辑。
    def song(self):
        return self._fake_song

    # 函数作用：执行 application 相关逻辑。
    def application(self):
        return getattr(self, "_fake_application", FakeApplication())

    # 函数作用：执行 log message 相关逻辑。
    def log_message(self, message):
        if not hasattr(self, "messages"):
            self.messages = []
        self.messages.append(("log", message))

    # 函数作用：执行 show message 相关逻辑。
    def show_message(self, message):
        if not hasattr(self, "messages"):
            self.messages = []
        self.messages.append(("show", message))

    # 函数作用：执行 schedule message 相关逻辑。
    def schedule_message(self, _delay, callback):
        callback()

    # 函数作用：执行 disconnect 相关逻辑。
    def disconnect(self):
        self.messages.append(("disconnect", "called"))


# 类作用：封装 Fake Browser Item 的状态和行为。
class FakeBrowserItem:
    # 函数作用：执行 init 相关逻辑。
    def __init__(self, name, is_folder=False, is_loadable=False, uri=None, children=None):
        self.name = name
        self.is_folder = is_folder
        self.is_loadable = is_loadable
        self.uri = uri
        self._children = list(children or [])

    @property
    # 函数作用：执行 iter children 相关逻辑。
    def iter_children(self):
        return tuple(self._children)


# 类作用：封装 Fake Browser 的状态和行为。
class FakeBrowser:
    # 函数作用：执行 init 相关逻辑。
    def __init__(self):
        self.sounds = (
            FakeBrowserItem("Cold Synths", is_folder=True, uri="browser://sounds/cold"),
            FakeBrowserItem("Glass Pad", is_loadable=True, uri="browser://sounds/glass"),
        )
        self.instruments = (FakeBrowserItem("Wavetable", is_loadable=True),)


# 类作用：封装 Fake Application 的状态和行为。
class FakeApplication:
    # 函数作用：执行 init 相关逻辑。
    def __init__(self, browser=None):
        self.browser = browser or FakeBrowser()


# 函数作用：加载 bridge module。
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


# 类作用：封装 Nina Rust Bridge Tests 的状态和行为。
class NinaRustBridgeTests(unittest.TestCase):
    # 函数作用：验证 bridge constants use localhost and expected name 场景。
    def test_bridge_constants_use_localhost_and_expected_name(self):
        module = load_bridge_module()

        self.assertEqual(module.BRIDGE_NAME, "NinaRustBridge")
        self.assertEqual(module.HOST, "127.0.0.1")
        self.assertEqual(module.PORT, 9878)

    # 函数作用：验证 health check returns bridge identity 场景。
    def test_health_check_returns_bridge_identity(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident

        response = bridge._process_command({"type": "health_check", "params": {"ping": "pong"}})

        self.assertEqual(response["status"], "success")
        self.assertEqual(response["result"]["ok"], True)
        self.assertEqual(response["result"]["name"], "NinaRustBridge")
        self.assertEqual(response["result"]["echo"], {"ping": "pong"})

    # 函数作用：验证 unknown command returns error response 场景。
    def test_unknown_command_returns_error_response(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident

        response = bridge._process_command({"type": "does_not_exist", "params": {}})

        self.assertEqual(response["status"], "error")
        self.assertIn("Unknown NinaRustBridge command", response["message"])

    # 函数作用：验证 snapshot live set summarizes fake song 场景。
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

    # 函数作用：验证 transport commands return requested state even if live updates later 场景。
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

    # 函数作用：验证 create midi clip range creates arrangement clip on midi track 场景。
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

    # 函数作用：验证 create midi clip range rejects audio track 场景。
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

    # 函数作用：验证 write midi clip sets notes on created arrangement clip 场景。
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

    # 函数作用：验证 export midi track returns all arrangement clip notes on track timeline 场景。
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

    # 函数作用：验证 create midi track inserts named track 场景。
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

    # 函数作用：验证 create audio track inserts named audio track 场景。
    def test_create_audio_track_inserts_named_audio_track(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident
        bridge._song = FakeSong()

        response = bridge._process_command(
            {
                "type": "create_audio_track",
                "params": {"index": 2, "name": "Printed Stems"},
            }
        )

        self.assertEqual(response["status"], "success")
        self.assertEqual(response["result"]["index"], 2)
        self.assertEqual(response["result"]["name"], "Printed Stems")
        self.assertEqual(response["result"]["track_count"], 4)
        self.assertEqual(bridge._song.tracks[2].name, "Printed Stems")
        self.assertEqual(bridge._song.tracks[2].has_audio_input, True)
        self.assertEqual(bridge._song.tracks[2].has_midi_input, False)

    # 函数作用：验证 audio import clip places file on audio track 场景。
    def test_audio_import_clip_places_file_on_audio_track(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident
        bridge._song = FakeSong()

        response = bridge._process_command(
            {
                "type": "audio_import_clip",
                "params": {
                    "track_index": 2,
                    "file_path": "/tmp/nina-loop.wav",
                    "destination_time": 8.0,
                    "name": "Loop Print",
                },
            }
        )

        self.assertEqual(response["status"], "success")
        self.assertEqual(response["result"]["track_name"], "Vocal")
        self.assertEqual(response["result"]["file_path"], "/tmp/nina-loop.wav")
        self.assertEqual(response["result"]["destination_time"], 8.0)
        self.assertEqual(response["result"]["clip_name"], "Loop Print")
        self.assertEqual(response["result"]["is_audio_clip"], True)
        self.assertEqual(len(bridge._song.tracks[2].arrangement_clips), 1)

    # 函数作用：验证 audio import clip rejects midi track 场景。
    def test_audio_import_clip_rejects_midi_track(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident
        bridge._song = FakeSong()

        response = bridge._process_command(
            {
                "type": "audio_import_clip",
                "params": {
                    "track_index": 0,
                    "file_path": "/tmp/nina-loop.wav",
                    "destination_time": 0.0,
                },
            }
        )

        self.assertEqual(response["status"], "error")
        self.assertIn("not an audio track", response["message"])

    # 函数作用：验证 audio context combines audio clips and effect chain 场景。
    def test_audio_context_combines_audio_clips_and_effect_chain(self):
        module = load_bridge_module()
        bridge = module.NinaRustBridge.__new__(module.NinaRustBridge)
        bridge._main_thread_id = threading.current_thread().ident
        bridge._song = FakeSong()
        bridge._song.tracks[2].devices = [FakeDevice("Hybrid Reverb", "HybridReverb")]
        clip = FakeArrangementClip(16.0, 32.0)
        clip.name = "verse.wav"
        clip.file_path = "/tmp/verse.wav"
        clip.is_midi_clip = False
        clip.is_audio_clip = True
        bridge._song.tracks[2].arrangement_clips = [clip]

        response = bridge._process_command(
            {"type": "audio_context", "params": {"track_index": 2}}
        )

        self.assertEqual(response["status"], "success")
        self.assertEqual(response["result"]["track"]["name"], "Vocal")
        self.assertEqual(response["result"]["clip_count"], 1)
        self.assertEqual(response["result"]["clips"][0]["file_path"], "/tmp/verse.wav")
        self.assertEqual(response["result"]["effect_count"], 1)
        self.assertEqual(response["result"]["effects"][0]["name"], "Hybrid Reverb")

    # 函数作用：验证 browser scan root returns first level items 场景。
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

    # 函数作用：验证 device scan track returns device chain overview 场景。
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

    # 函数作用：验证 device scan track can include parameter values 场景。
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

    # 函数作用：验证 drum scan track returns used pad note map 场景。
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
