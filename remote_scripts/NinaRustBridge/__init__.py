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
