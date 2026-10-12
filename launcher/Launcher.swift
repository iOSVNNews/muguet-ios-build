// muguet - Launcher.swift
// Constantin Clerc (c22dev) - 0.1

import AppKit
import SwiftUI
import UniformTypeIdentifiers

let supportURL = FileManager.default.homeDirectoryForCurrentUser.appendingPathComponent("Library/Application Support/Muguet")
let configURL = supportURL.appendingPathComponent("config.json")
let gameURL = supportURL.appendingPathComponent("game")
let gameBinary = Bundle.main.bundleURL.appendingPathComponent("Contents/MacOS/muguet")

func gameReady() -> Bool {
    let fm = FileManager.default
    guard fm.fileExists(atPath: gameURL.appendingPathComponent("data/manifest.json").path),
          let builds = try? fm.contentsOfDirectory(at: gameURL.appendingPathComponent("b"), includingPropertiesForKeys: nil) else { return false }
    let embedded = Bundle.main.object(forInfoDictionaryKey: "MuguetEmbeddedCode") as? Bool ?? false
    return builds.contains { (embedded || fm.fileExists(atPath: $0.appendingPathComponent("game.wasm").path)) && fm.fileExists(atPath: $0.appendingPathComponent("shaders/index.json").path) }
}

final class Importer: ObservableObject {
    @Published var ready = gameReady()
    @Published var progress: Double?
    @Published var error: String?

    func choose() {
        let panel = NSOpenPanel()
        panel.allowedContentTypes = [.zip]
        panel.message = "Choose your game data .zip"
        guard panel.runModal() == .OK, let url = panel.url else { return }
        run(url)
    }

    func run(_ zip: URL) {
        error = nil
        progress = 0
        let p = Process()
        p.executableURL = gameBinary
        p.arguments = ["--import", zip.path]
        let out = Pipe(), err = Pipe()
        p.standardOutput = out
        p.standardError = err
        out.fileHandleForReading.readabilityHandler = { h in
            guard let text = String(data: h.availableData, encoding: .utf8) else { return }
            for line in text.split(separator: "\n") {
                let f = line.split(separator: " ")
                if f.count == 3, f[0] == "progress", let d = Double(f[1]), let t = Double(f[2]), t > 0 {
                    DispatchQueue.main.async { self.progress = d / t }
                }
            }
        }
        p.terminationHandler = { p in
            out.fileHandleForReading.readabilityHandler = nil
            let message = String(data: err.fileHandleForReading.readDataToEndOfFile(), encoding: .utf8) ?? ""
            DispatchQueue.main.async {
                self.progress = nil
                self.ready = gameReady()
                if p.terminationStatus != 0 || !self.ready {
                    self.error = "Import failed: " + (message.split(separator: "\n").last.map(String.init) ?? "unknown error")
                }
            }
        }
        do { try p.run() } catch { progress = nil; self.error = "Could not start the import: \(error.localizedDescription)" }
    }
}

final class Settings: ObservableObject {
    @Published var mode = 2
    @Published var scale = 0.5
    @Published var fullscreen = true
    @Published var fps = 60
    @Published var mouse = 2.0
    @Published var mute = false
    @Published var multiplayer = false
    @Published var host = true
    @Published var server = ""
    @Published var name = NSUserName()
    var extra: [String: Any] = [:]

    init() {
        guard let data = try? Data(contentsOf: configURL),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else { return }
        extra = json
        if let v = json["start_mode"] as? Int, (1...2).contains(v) { mode = v }
        if let v = json["scale"] as? Double { scale = v }
        if let v = json["fullscreen"] as? Bool { fullscreen = v }
        if let v = json["fps_limit"] as? Int { fps = v }
        if let v = json["mouse_sensitivity"] as? Double { mouse = v }
        if let v = json["mute"] as? Bool { mute = v }
        if let v = json["multiplayer"] as? Bool { multiplayer = v }
        if let v = json["host"] as? Bool { host = v }
        if let v = json["server"] as? String { server = v }
        if let v = json["player_name"] as? String, !v.isEmpty { name = v }
    }

    func save() throws {
        var json = extra
        json["start_mode"] = mode
        json["scale"] = scale
        json["fullscreen"] = fullscreen
        json["fps_limit"] = fps
        json["mouse_sensitivity"] = (mouse * 10).rounded() / 10
        json["mute"] = mute
        json["multiplayer"] = multiplayer
        json["host"] = host
        json["server"] = server.trimmingCharacters(in: .whitespaces)
        json["player_name"] = name.trimmingCharacters(in: .whitespaces)
        if json["extra_args"] == nil { json["extra_args"] = [String]() }
        try FileManager.default.createDirectory(at: configURL.deletingLastPathComponent(), withIntermediateDirectories: true)
        let data = try JSONSerialization.data(withJSONObject: json, options: [.prettyPrinted, .sortedKeys])
        try data.write(to: configURL)
    }
}

struct LauncherView: View {
    @StateObject private var s = Settings()
    @StateObject private var importer = Importer()
    @State private var error: String?

    var body: some View {
        VStack(spacing: 18) {
            Image(nsImage: NSApp.applicationIconImage).resizable().scaledToFit().frame(height: 110)
            if !importer.ready || importer.progress != nil {
                importView
            } else {
                settingsView
            }
            Text("proudly made by c22dev")
                .font(.caption).foregroundStyle(.secondary)
        }
        .padding(24)
        .frame(width: 480)
        .fixedSize(horizontal: false, vertical: true)
    }

    @ViewBuilder var importView: some View {
        VStack(spacing: 12) {
            if let p = importer.progress {
                ProgressView(value: p)
                Text("Importing… \(Int(p * 100)) %").font(.callout).monospacedDigit()
            } else {
                Button("Import game data .zip") { importer.choose() }
                    .controlSize(.large)
                    .buttonStyle(.borderedProminent)
            }
            if let e = importer.error { Text(e).foregroundStyle(.red).font(.callout).multilineTextAlignment(.center) }
        }
    }

    @ViewBuilder var settingsView: some View {
        VStack(spacing: 18) {
            Form {
                Picker("Mode", selection: $s.mode) {
                    Text("Story Mode").tag(1)
                    Text("Sandbox").tag(2)
                }
                Picker("Resolution", selection: $s.scale) {
                    Text("50 % (fastest)").tag(0.5)
                    Text("75 %").tag(0.75)
                    Text("100 % (sharpest)").tag(1.0)
                }
                Picker("Frame limit", selection: $s.fps) {
                    Text("30 fps (cooler)").tag(30)
                    Text("60 fps").tag(60)
                    Text("Uncapped").tag(0)
                }
                HStack {
                    Text("Mouse sensitivity")
                    Slider(value: $s.mouse, in: 0.5...6, step: 0.5)
                    Text(String(format: "%.1f×", s.mouse)).monospacedDigit().frame(width: 40, alignment: .trailing)
                }
                Toggle("Fullscreen", isOn: $s.fullscreen)
                Toggle("Mute audio", isOn: $s.mute)
                Section("Multiplayer") {
                    Toggle("Play with friends", isOn: $s.multiplayer)
                    if s.multiplayer {
                        TextField("Your name", text: $s.name)
                        Picker("Session", selection: $s.host) {
                            Text("Host").tag(true)
                            Text("Join").tag(false)
                        }
                        .pickerStyle(.segmented)
                        if s.host {
                            Text("Friends join with: \(localIPs().joined(separator: " or "))")
                                .font(.callout).textSelection(.enabled)
                            Text("Over the internet, forward UDP port 47520 to this Mac.")
                                .font(.caption).foregroundStyle(.secondary)
                        } else {
                            TextField("Host address", text: $s.server, prompt: Text("192.168.1.20"))
                        }
                    }
                }
            }
            .formStyle(.grouped)
            .scrollDisabled(true)
            .fixedSize(horizontal: false, vertical: true)
            if let error { Text(error).foregroundStyle(.red).font(.callout) }
            Button(action: play) {
                Text("Play").font(.title3.weight(.semibold)).frame(maxWidth: .infinity)
            }
            .controlSize(.large)
            .buttonStyle(.borderedProminent)
            .keyboardShortcut(.defaultAction)
            Text("In game: Cmd+F fullscreen · Cmd+any key releases the mouse · Cmd+Q quits")
                .font(.caption).foregroundStyle(.secondary)
            Text("The first start compiles the game for this Mac (about a minute).")
                .font(.caption).foregroundStyle(.secondary)
            Button("Import other game data…") { importer.choose() }
                .buttonStyle(.link).font(.caption)
        }
    }

    func play() {
        if s.multiplayer && !s.host && s.server.trimmingCharacters(in: .whitespaces).isEmpty {
            error = "Enter the host's address to join."
            return
        }
        do { try s.save() } catch { self.error = "Could not save settings: \(error.localizedDescription)"; return }

        let game = gameBinary.path
        var argv: [UnsafeMutablePointer<CChar>?] = [strdup(game), nil]
        execv(game, &argv)
        error = "Could not start the game (\(String(cString: strerror(errno))))"
    }
}

func localIPs() -> [String] {
    var out: [String] = []
    var ifs: UnsafeMutablePointer<ifaddrs>?
    guard getifaddrs(&ifs) == 0, let first = ifs else { return ["?"] }
    defer { freeifaddrs(ifs) }
    for p in sequence(first: first, next: { $0.pointee.ifa_next }) {
        guard let a = p.pointee.ifa_addr, a.pointee.sa_family == UInt8(AF_INET) else { continue }
        var host = [CChar](repeating: 0, count: Int(NI_MAXHOST))
        if getnameinfo(a, socklen_t(a.pointee.sa_len), &host, socklen_t(host.count), nil, 0, NI_NUMERICHOST) == 0 {
            let ip = String(cString: host)
            if !ip.hasPrefix("127.") && !out.contains(ip) { out.append(ip) }
        }
    }
    return out.isEmpty ? ["?"] : out
}

@main
struct MuguetLauncher: App {
    var body: some Scene {
        WindowGroup("Muguet") { LauncherView() }
            .windowResizability(.contentSize)
    }
}
