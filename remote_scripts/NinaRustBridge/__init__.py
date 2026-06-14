# -*- coding: utf-8 -*-
"""Ableton Live Remote Script bridge for the Nina Rust CLI.

This file is loaded by Ableton Live's embedded Python runtime. Keep it thin:
it exposes a localhost TCP/JSON bridge and leaves music decisions to Rust.
"""

from __future__ import absolute_import, print_function

import json
import socket
import threading
import time
import traceback

from _Framework.ControlSurface import ControlSurface

try:
    import Queue as queue
except ImportError:
    import queue


BRIDGE_NAME = "NinaRustBridge"
HOST = "127.0.0.1"
PORT = 9878
RECV_BYTES = 8192
MAIN_THREAD_TIMEOUT_SECONDS = 15.0


def create_instance(c_instance):
    return NinaRustBridge(c_instance)


class NinaRustBridge(ControlSurface):
    """Thin Ableton Control Surface adapter for localhost JSON commands."""

    def __init__(self, c_instance):
        ControlSurface.__init__(self, c_instance)
        self._server = None
        self._server_thread = None
        self._client_threads = []
        self._running = False
        self._main_thread_id = threading.current_thread().ident
        self._song = self.song()
        self.log_message("{0} initializing".format(BRIDGE_NAME))
        self._start_server()

    def disconnect(self):
        self._running = False
        if self._server is not None:
            try:
                self._server.close()
            except Exception:
                pass
            self._server = None
        ControlSurface.disconnect(self)

    def _start_server(self):
        try:
            self._server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self._server.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
            self._server.bind((HOST, PORT))
            self._server.listen(5)
            self._server.settimeout(1.0)
            self._running = True
            self._server_thread = threading.Thread(target=self._server_loop)
            self._server_thread.daemon = True
            self._server_thread.start()
            message = "{0} listening on {1}:{2}".format(BRIDGE_NAME, HOST, PORT)
            self.log_message(message)
            self.show_message(message)
        except Exception as exc:
            self.log_message("{0} server start failed: {1}".format(BRIDGE_NAME, exc))
            self.log_message(traceback.format_exc())
            self.show_message("{0} error: {1}".format(BRIDGE_NAME, exc))

    def _server_loop(self):
        while self._running:
            try:
                client, address = self._server.accept()
                self.log_message("{0} client connected: {1}".format(BRIDGE_NAME, address))
                thread = threading.Thread(target=self._handle_client, args=(client,))
                thread.daemon = True
                thread.start()
                self._client_threads.append(thread)
                self._client_threads = [item for item in self._client_threads if item.is_alive()]
            except socket.timeout:
                continue
            except Exception as exc:
                if self._running:
                    self.log_message("{0} accept error: {1}".format(BRIDGE_NAME, exc))
                time.sleep(0.2)

    def _handle_client(self, client):
        buffer = ""
        client.settimeout(None)
        try:
            while self._running:
                data = client.recv(RECV_BYTES)
                if not data:
                    break
                try:
                    buffer += data.decode("utf-8")
                except AttributeError:
                    buffer += data
                buffer = self._process_client_buffer(client, buffer)
        except Exception as exc:
            self.log_message("{0} client error: {1}".format(BRIDGE_NAME, exc))
            self.log_message(traceback.format_exc())
        finally:
            try:
                client.close()
            except Exception:
                pass

    def _process_client_buffer(self, client, buffer):
        while buffer:
            stripped = buffer.lstrip()
            if not stripped:
                return ""
            if "\n" in stripped:
                line, remainder = stripped.split("\n", 1)
                if not line.strip():
                    buffer = remainder
                    continue
                command = json.loads(line)
                self._send_response(client, self._process_command(command))
                buffer = remainder
                continue
            try:
                command = json.loads(stripped)
            except ValueError:
                return stripped
            self._send_response(client, self._process_command(command))
            return ""
        return buffer

    def _send_response(self, client, response):
        payload = json.dumps(response) + "\n"
        try:
            client.sendall(payload.encode("utf-8"))
        except AttributeError:
            client.sendall(payload)

    def _process_command(self, command):
        command_type = command.get("type") or command.get("method") or ""
        params = command.get("params", {}) or {}
        handlers = {
            "health_check": self._health_check,
            "snapshot_live_set": self._snapshot_live_set,
            "set_tempo": self._set_tempo,
            "start_playback": self._start_playback,
            "stop_playback": self._stop_playback,
            "create_midi_track": self._create_midi_track,
            "create_midi_clip_range": self._create_midi_clip_range,
            "write_midi_clip": self._write_midi_clip,
            "browser_scan_root": self._browser_scan_root,
            "device_scan_track": self._device_scan_track,
            "drum_scan_track": self._drum_scan_track,
            "export_midi_track": self._export_midi_track,
        }
        if command_type not in handlers:
            return {
                "status": "error",
                "message": "Unknown NinaRustBridge command: {0}".format(command_type),
            }
        try:
            result = self._run_on_main_thread(lambda: handlers[command_type](params))
            return {"status": "success", "result": result}
        except Exception as exc:
            self.log_message("{0} command failed: {1}: {2}".format(BRIDGE_NAME, command_type, exc))
            self.log_message(traceback.format_exc())
            return {"status": "error", "message": str(exc)}

    def _run_on_main_thread(self, func):
        if threading.current_thread().ident == getattr(self, "_main_thread_id", None):
            return func()
        result_queue = queue.Queue()

        def task():
            try:
                result_queue.put(("ok", func()))
            except Exception as exc:
                result_queue.put(("error", exc))

        self.schedule_message(0, task)
        status, value = result_queue.get(timeout=MAIN_THREAD_TIMEOUT_SECONDS)
        if status == "error":
            raise value
        return value

    def _health_check(self, params):
        return {
            "ok": True,
            "name": BRIDGE_NAME,
            "host": HOST,
            "port": PORT,
            "echo": params,
        }

    def _snapshot_live_set(self, _params):
        tracks = []
        for index, track in enumerate(self._song.tracks):
            tracks.append(self._track_summary(index, track))
        return {
            "tempo": float(self._song.tempo),
            "signature_numerator": int(self._song.signature_numerator),
            "signature_denominator": int(self._song.signature_denominator),
            "is_playing": bool(self._song.is_playing),
            "track_count": len(self._song.tracks),
            "scene_count": len(self._song.scenes),
            "tracks": tracks,
        }

    def _set_tempo(self, params):
        tempo = float(params.get("tempo", 120.0))
        if tempo <= 0.0:
            raise ValueError("tempo must be positive")
        self._song.tempo = tempo
        return {"tempo": float(self._song.tempo)}

    def _start_playback(self, _params):
        self._song.start_playing()
        return {"is_playing": True}

    def _stop_playback(self, _params):
        self._song.stop_playing()
        return {"is_playing": False}

    def _create_midi_track(self, params):
        index = params.get("index")
        if index is None:
            index = len(self._song.tracks)
        index = int(index)
        if index < 0 or index > len(self._song.tracks):
            raise IndexError("Track index out of range: {0}".format(index))

        self._song.create_midi_track(index)
        track = self._song.tracks[index]
        name = params.get("name")
        if name is not None:
            try:
                track.name = str(name)
            except Exception:
                pass
        return {
            "index": index,
            "name": getattr(track, "name", ""),
            "has_midi_input": bool(getattr(track, "has_midi_input", False)),
            "has_audio_input": bool(getattr(track, "has_audio_input", False)),
            "track_count": len(self._song.tracks),
        }

    def _create_midi_clip_range(self, params):
        track_index = int(params.get("track_index", 0))
        start_bar = int(params.get("start_bar", 1))
        end_bar = int(params.get("end_bar", start_bar + 1))
        name = params.get("name")
        track = self._require_midi_track(track_index)
        start_time, length = self._bar_range_to_beats(start_bar, end_bar)

        created_clip = track.create_midi_clip(start_time, length)
        clip = self._resolve_created_arrangement_clip(track, created_clip, start_time)
        if name is not None and clip is not None:
            try:
                clip.name = str(name)
            except Exception:
                pass

        return self._midi_clip_range_result(
            track_index,
            track,
            clip,
            start_bar,
            end_bar,
            start_time,
            length,
        )

    def _write_midi_clip(self, params):
        notes = params.get("notes") or []
        result = self._create_midi_clip_range(params)
        track = self._song.tracks[int(params.get("track_index", 0))]
        clip = self._find_arrangement_clip_at(track, result["start_time"])
        if clip is None:
            raise ValueError("Created MIDI clip could not be resolved")
        if not bool(getattr(clip, "is_midi_clip", False)):
            raise ValueError("Created clip is not a MIDI clip")

        live_notes = []
        for note in notes:
            live_notes.append(self._coerce_midi_note(note))
        clip.set_notes(tuple(live_notes))

        return {"clip": result, "note_count": len(live_notes)}

    def _browser_scan_root(self, params):
        root_name = str(params.get("root") or "sounds")
        limit = int(params.get("limit") or 25)
        if limit <= 0:
            raise ValueError("limit must be positive")
        application = self.application()
        browser = getattr(application, "browser", None)
        if browser is None:
            raise ValueError("Ableton browser is not available")
        if not hasattr(browser, root_name):
            raise ValueError("Unknown browser root: {0}".format(root_name))

        root = getattr(browser, root_name)
        items = []
        truncated = False
        for item in self._iter_browser_root(root):
            if len(items) >= limit:
                truncated = True
                break
            items.append(self._browser_item_summary(root_name, item))

        return {
            "root": root_name,
            "count": len(items),
            "truncated": truncated,
            "items": items,
        }

    def _device_scan_track(self, params):
        track_index = int(params.get("track_index", 0))
        include_parameters = bool(params.get("include_parameters", False))
        track = self._track_at_index(track_index)
        devices = []
        for index, device in enumerate(getattr(track, "devices", [])):
            devices.append(
                self._device_summary(
                    index, device, depth=0, include_parameters=include_parameters
                )
            )
        return {
            "track": {
                "index": track_index,
                "name": getattr(track, "name", ""),
                "has_midi_input": bool(getattr(track, "has_midi_input", False)),
                "has_audio_input": bool(getattr(track, "has_audio_input", False)),
            },
            "devices": devices,
            "device_count": len(devices),
        }

    def _drum_scan_track(self, params):
        track_index = int(params.get("track_index", 0))
        include_empty_pads = bool(params.get("include_empty_pads", False))
        track = self._track_at_index(track_index)
        racks = []
        for index, device in enumerate(getattr(track, "devices", []) or []):
            if self._is_drum_rack_device(device):
                racks.append(self._drum_rack_summary(index, device, include_empty_pads))
        return {
            "track": {
                "index": track_index,
                "name": getattr(track, "name", ""),
                "has_midi_input": bool(getattr(track, "has_midi_input", False)),
                "has_audio_input": bool(getattr(track, "has_audio_input", False)),
            },
            "rack_count": len(racks),
            "racks": racks,
        }

    def _export_midi_track(self, params):
        track_index = int(params.get("track_index", 0))
        track = self._require_midi_track(track_index)
        clips = []
        notes = []

        for clip_index, clip in enumerate(self._iter_arrangement_midi_clips(track)):
            clip_notes = self._read_midi_clip_notes(clip)
            start_time = self._safe_float(getattr(clip, "start_time", 0.0)) or 0.0
            length = self._safe_float(getattr(clip, "length", 0.0)) or 0.0
            for note in clip_notes:
                shifted = dict(note)
                shifted["start"] = start_time + float(note.get("start", 0.0))
                notes.append(shifted)
            clips.append(
                {
                    "index": clip_index,
                    "name": self._safe_optional_text(getattr(clip, "name", None)),
                    "start_time": float(start_time),
                    "length": float(length),
                    "note_count": len(clip_notes),
                }
            )

        notes.sort(key=lambda note: (float(note.get("start", 0.0)), int(note.get("pitch", 0))))
        if clips:
            start_beat = min(float(clip.get("start_time", 0.0)) for clip in clips)
            end_beat = max(
                float(clip.get("start_time", 0.0)) + float(clip.get("length", 0.0))
                for clip in clips
            )
        elif notes:
            start_beat = min(float(note.get("start", 0.0)) for note in notes)
            end_beat = max(
                float(note.get("start", 0.0)) + float(note.get("duration", 0.0))
                for note in notes
            )
        else:
            start_beat = 0.0
            end_beat = 0.0

        return {
            "track": {
                "index": track_index,
                "name": getattr(track, "name", ""),
                "has_midi_input": bool(getattr(track, "has_midi_input", False)),
                "has_audio_input": bool(getattr(track, "has_audio_input", False)),
            },
            "clip_count": len(clips),
            "note_count": len(notes),
            "start_beat": float(start_beat),
            "end_beat": float(end_beat),
            "clips": clips,
            "notes": notes,
        }

    def _track_summary(self, index, track):
        return {
            "index": index,
            "name": track.name,
            "has_midi_input": bool(track.has_midi_input),
            "has_audio_input": bool(track.has_audio_input),
            "device_count": len(track.devices),
            "clip_slot_count": len(track.clip_slots),
        }

    def _track_at_index(self, track_index):
        if track_index < 0 or track_index >= len(self._song.tracks):
            raise IndexError("Track index out of range: {0}".format(track_index))
        return self._song.tracks[track_index]

    def _require_midi_track(self, track_index):
        track = self._track_at_index(track_index)
        if not bool(getattr(track, "has_midi_input", False)):
            raise ValueError(
                "Track {0} '{1}' is not a MIDI track".format(
                    track_index, getattr(track, "name", "")
                )
            )
        return track

    def _bar_range_to_beats(self, start_bar, end_bar):
        if start_bar < 1:
            raise ValueError("start_bar must be at least 1")
        if end_bar <= start_bar:
            raise ValueError("end_bar must be greater than start_bar")
        beats_per_bar = int(getattr(self._song, "signature_numerator", 4))
        if beats_per_bar <= 0:
            raise ValueError("song signature_numerator must be positive")
        start_time = float((start_bar - 1) * beats_per_bar)
        length = float((end_bar - start_bar) * beats_per_bar)
        return start_time, length

    def _resolve_created_arrangement_clip(self, track, created_clip, start_time):
        if created_clip is not None:
            return created_clip
        return self._find_arrangement_clip_at(track, start_time)

    def _find_arrangement_clip_at(self, track, start_time):
        for clip in getattr(track, "arrangement_clips", []):
            try:
                if abs(float(clip.start_time) - float(start_time)) < 0.01:
                    return clip
            except Exception:
                continue
        return None

    def _iter_arrangement_midi_clips(self, track):
        clips = []
        for clip in getattr(track, "arrangement_clips", []) or []:
            if bool(getattr(clip, "is_midi_clip", False)):
                clips.append(clip)
        clips.sort(key=lambda clip: self._safe_float(getattr(clip, "start_time", 0.0)) or 0.0)
        return clips

    def _midi_clip_range_result(
        self,
        track_index,
        track,
        clip,
        start_bar,
        end_bar,
        start_time,
        length,
    ):
        clip_name = None
        is_midi_clip = True
        if clip is not None:
            clip_name = getattr(clip, "name", None)
            is_midi_clip = bool(getattr(clip, "is_midi_clip", True))
        return {
            "track_index": track_index,
            "track_name": getattr(track, "name", ""),
            "start_bar": start_bar,
            "end_bar": end_bar,
            "start_time": float(start_time),
            "length": float(length),
            "clip_name": clip_name,
            "is_midi_clip": is_midi_clip,
        }

    def _coerce_midi_note(self, note):
        pitch = int(note.get("pitch", 60))
        start = float(note.get("start", note.get("start_time", 0.0)))
        duration = float(note.get("duration", 0.25))
        velocity = int(note.get("velocity", 100))
        mute = bool(note.get("mute", False))
        return (pitch, start, duration, velocity, mute)

    def _read_midi_clip_notes(self, clip):
        raw_notes = self._try_read_midi_clip_notes(clip)
        notes = []
        for raw_note in raw_notes:
            note = self._midi_note_summary(raw_note)
            if note is not None:
                notes.append(note)
        notes.sort(key=lambda note: (float(note.get("start", 0.0)), int(note.get("pitch", 0))))
        return notes

    def _try_read_midi_clip_notes(self, clip):
        length = self._safe_float(getattr(clip, "length", 0.0)) or 0.0
        attempts = (
            ("get_notes_extended", (0, 128, 0.0, length)),
            ("get_notes_extended", (0.0, 0, length, 128)),
            ("get_notes", (0.0, 0, length, 128)),
            ("get_notes", (0, 128, 0.0, length)),
        )
        for method_name, args in attempts:
            method = getattr(clip, method_name, None)
            if callable(method):
                try:
                    return self._normalize_note_collection(method(*args))
                except Exception:
                    continue
        return self._normalize_note_collection(getattr(clip, "notes", ()))

    def _normalize_note_collection(self, raw_notes):
        if raw_notes is None:
            return []
        if isinstance(raw_notes, dict):
            if "notes" in raw_notes:
                return raw_notes.get("notes") or []
            return list(raw_notes.values())
        return raw_notes

    def _midi_note_summary(self, raw_note):
        if isinstance(raw_note, dict):
            pitch = self._safe_int(raw_note.get("pitch"))
            start = self._safe_float(raw_note.get("start", raw_note.get("start_time")))
            duration = self._safe_float(raw_note.get("duration"))
            velocity = self._safe_int(raw_note.get("velocity"))
            mute = bool(raw_note.get("mute", raw_note.get("muted", False)))
            return self._build_note_summary(pitch, start, duration, velocity, mute)
        if isinstance(raw_note, (list, tuple)):
            if len(raw_note) < 4:
                return None
            pitch = self._safe_int(raw_note[0])
            start = self._safe_float(raw_note[1])
            duration = self._safe_float(raw_note[2])
            velocity = self._safe_int(raw_note[3])
            mute = bool(raw_note[4]) if len(raw_note) > 4 else False
            return self._build_note_summary(pitch, start, duration, velocity, mute)

        pitch = self._safe_int(getattr(raw_note, "pitch", None))
        start = self._safe_float(
            getattr(raw_note, "start_time", getattr(raw_note, "start", None))
        )
        duration = self._safe_float(getattr(raw_note, "duration", None))
        velocity = self._safe_int(getattr(raw_note, "velocity", None))
        mute = bool(getattr(raw_note, "mute", getattr(raw_note, "muted", False)))
        return self._build_note_summary(pitch, start, duration, velocity, mute)

    def _build_note_summary(self, pitch, start, duration, velocity, mute):
        if pitch is None or start is None or duration is None or velocity is None:
            return None
        return {
            "pitch": int(pitch),
            "start": float(start),
            "duration": float(duration),
            "velocity": int(velocity),
            "mute": bool(mute),
        }

    def _iter_browser_root(self, root):
        if hasattr(root, "iter_children"):
            try:
                return iter(root.iter_children)
            except Exception:
                return iter(())
        try:
            return iter(root)
        except Exception:
            return iter(())

    def _browser_item_summary(self, root_name, item):
        name = str(getattr(item, "name", ""))
        uri = getattr(item, "uri", None)
        if uri == "":
            uri = None
        return {
            "name": name,
            "path": "{0}/{1}".format(root_name, name),
            "is_folder": bool(getattr(item, "is_folder", False)),
            "is_loadable": bool(getattr(item, "is_loadable", False)),
            "uri": uri,
        }

    def _device_summary(self, index, device, depth, include_parameters):
        raw_parameters = getattr(device, "parameters", []) or []
        parameters = []
        if include_parameters:
            for parameter_index, parameter in enumerate(raw_parameters):
                parameters.append(self._device_parameter_summary(parameter_index, parameter))

        chains = []
        if depth < 3:
            for chain_index, chain in enumerate(getattr(device, "chains", []) or []):
                chains.append(
                    self._device_chain_summary(
                        chain_index, chain, depth + 1, include_parameters
                    )
                )

        class_name = self._safe_text(getattr(device, "class_name", None))
        return {
            "index": index,
            "name": self._safe_text(getattr(device, "name", "")),
            "class_name": class_name,
            "role": self._device_role(device, class_name, len(chains)),
            "is_rack": len(chains) > 0 or "GroupDevice" in class_name,
            "parameter_count": len(raw_parameters),
            "chain_count": len(chains),
            "parameters": parameters,
            "chains": chains,
        }

    def _device_chain_summary(self, index, chain, depth, include_parameters):
        devices = []
        for device_index, device in enumerate(getattr(chain, "devices", []) or []):
            devices.append(
                self._device_summary(
                    device_index,
                    device,
                    depth=depth,
                    include_parameters=include_parameters,
                )
            )
        return {
            "index": index,
            "name": self._safe_text(getattr(chain, "name", "")),
            "device_count": len(devices),
            "devices": devices,
        }

    def _drum_rack_summary(self, device_index, device, include_empty_pads):
        raw_pads = getattr(device, "drum_pads", []) or []
        pads = []
        for pad_index, pad in enumerate(raw_pads):
            summary = self._drum_pad_summary(pad_index, pad)
            if include_empty_pads or self._is_used_drum_pad(summary):
                pads.append(summary)
        return {
            "device_index": device_index,
            "name": self._safe_text(getattr(device, "name", "")),
            "class_name": self._safe_text(getattr(device, "class_name", "")),
            "pad_count": len(raw_pads),
            "used_pad_count": len(pads),
            "pads": pads,
        }

    def _drum_pad_summary(self, index, pad):
        chains = []
        for chain_index, chain in enumerate(getattr(pad, "chains", []) or []):
            chains.append(self._drum_pad_chain_summary(chain_index, chain))
        note = self._safe_int(getattr(pad, "note", None))
        name = self._safe_text(getattr(pad, "name", ""))
        role_text = " ".join([name] + [item.get("name", "") for item in chains])
        return {
            "index": index,
            "name": name,
            "note": note,
            "note_name": self._midi_note_name(note),
            "role_guess": self._drum_role_guess(role_text),
            "chain_count": len(chains),
            "chains": chains,
        }

    def _drum_pad_chain_summary(self, index, chain):
        devices = []
        for device_index, device in enumerate(getattr(chain, "devices", []) or []):
            devices.append(self._drum_pad_device_summary(device_index, device))
        out_note = self._safe_int(getattr(chain, "out_note", None))
        return {
            "index": index,
            "name": self._safe_text(getattr(chain, "name", "")),
            "out_note": out_note,
            "out_note_name": self._midi_note_name(out_note),
            "device_count": len(devices),
            "devices": devices,
        }

    def _drum_pad_device_summary(self, index, device):
        class_name = self._safe_text(getattr(device, "class_name", ""))
        return {
            "index": index,
            "name": self._safe_text(getattr(device, "name", "")),
            "class_name": class_name,
            "role": self._device_role(device, class_name, 0),
        }

    def _device_parameter_summary(self, index, parameter):
        value = self._safe_float(getattr(parameter, "value", None))
        return {
            "index": index,
            "name": self._safe_text(getattr(parameter, "name", "")),
            "value": value,
            "min": self._safe_float(getattr(parameter, "min", None)),
            "max": self._safe_float(getattr(parameter, "max", None)),
            "display_value": self._parameter_display_value(parameter, value),
            "is_enabled": bool(getattr(parameter, "is_enabled", True)),
            "is_quantized": bool(getattr(parameter, "is_quantized", False)),
        }

    def _parameter_display_value(self, parameter, value):
        display = getattr(parameter, "str_for_value", None)
        if callable(display):
            try:
                return self._safe_optional_text(display(value))
            except Exception:
                return None
        return self._safe_optional_text(display)

    def _device_role(self, device, class_name, chain_count):
        device_type = self._safe_text(getattr(device, "type", ""))
        role_source = "{0} {1}".format(class_name, device_type).lower()
        if chain_count > 0 or "groupdevice" in role_source or "rack" in role_source:
            if "audio" in role_source:
                return "audio_effect_rack"
            if "midi" in role_source:
                return "midi_effect_rack"
            return "instrument_rack"
        if "midi" in role_source:
            return "midi_effect"
        if "instrument" in role_source or class_name in (
            "Operator",
            "Wavetable",
            "Simpler",
            "Sampler",
            "Collision",
            "Tension",
        ):
            return "instrument"
        return "audio_effect"

    def _is_drum_rack_device(self, device):
        class_name = self._safe_text(getattr(device, "class_name", ""))
        if "DrumGroupDevice" in class_name or "DrumRack" in class_name:
            return True
        return hasattr(device, "drum_pads")

    def _is_used_drum_pad(self, summary):
        return bool(summary.get("name")) or int(summary.get("chain_count", 0)) > 0

    def _drum_role_guess(self, text):
        lowered = text.lower()
        if "kick" in lowered or "bd" in lowered:
            return "kick"
        if "snare" in lowered or "sd" in lowered or "clap" in lowered or "rim" in lowered:
            return "snare"
        if "closed" in lowered and ("hat" in lowered or "hh" in lowered):
            return "closed_hat"
        if "open" in lowered and ("hat" in lowered or "hh" in lowered):
            return "open_hat"
        if "hat" in lowered or "hihat" in lowered or "hh" in lowered:
            return "hat"
        if "crash" in lowered:
            return "crash"
        if "ride" in lowered:
            return "ride"
        if "tom" in lowered:
            return "tom"
        return "unknown"

    def _midi_note_name(self, note):
        if note is None:
            return None
        names = ("C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B")
        try:
            value = int(note)
        except Exception:
            return None
        return "{0}{1}".format(names[value % 12], int(value / 12) - 2)

    def _safe_float(self, value):
        if value is None:
            return None
        try:
            return float(value)
        except Exception:
            return None

    def _safe_int(self, value):
        if value is None:
            return None
        try:
            return int(value)
        except Exception:
            return None

    def _safe_text(self, value):
        if value is None:
            return ""
        try:
            return str(value)
        except Exception:
            return ""

    def _safe_optional_text(self, value):
        text = self._safe_text(value)
        if text == "":
            return None
        return text
