// muguet - Launcher.swift
// Constantin Clerc (c22dev) - 0.1

import SwiftUI
import UIKit
import UniformTypeIdentifiers

let configURL = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
    .appendingPathComponent("config.json")

final class Settings: ObservableObject {
    @Published var mode = 2
    @Published var scale = 0.5
    @Published var fps = 60
    @Published var textures = 0
    @Published var touch = 1.5
    @Published var aimSensitivity = 0.8
    @Published var aimMode = "toggle"
    @Published var fireAim = true
    @Published var mute = false
    @Published var multiplayer = false
    @Published var host = false
    @Published var server = "18.143.103.253"
    @Published var name = "Player" + String(Int.random(in: 100...999))
    @Published var roomDigits = String(Int.random(in: 1000...9999))
    @Published var joinCode = ""
    @Published var roomPass = ""
    @Published var roomMode = "create" // "create", "join", "public", "lan"
    @Published var pvp = false
    @Published var promotionSync = false
    @Published var deepOptimization = false
    @Published var maxPlayers = 32
    @Published var buttonScale = 1.0
    @Published var buttonOffsetX = 0.0
    @Published var buttonOffsetY = 0.0
    @Published var buttonOpacity = 0.22
    @Published var stickOffsetX = 0.0
    @Published var stickOffsetY = 0.0
    @Published var customButtons: [Int: [Double]] = [:]
    var extra: [String: Any] = [:]

    func apply120FpsBoost() {
        fps = 120
        scale = 0.5
        textures = 0
        deepOptimization = true
        promotionSync = true
        do { try save() } catch {}
    }

    var formattedCreateCode: String {
        let m = mode == 1 ? "S" : "B"
        let p = pvp ? "X" : "P"
        return "\(m)\(p)-\(roomDigits)"
    }

    init() {
        guard let data = try? Data(contentsOf: configURL),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else { return }
        extra = json
        if let v = json["start_mode"] as? Int, (1...2).contains(v) { mode = v }
        if let v = json["scale"] as? Double { scale = v }
        if let v = json["fps_limit"] as? Int, [30, 60, 120].contains(v) { fps = v }
        if let v = json["max_players"] as? Int, [16, 32, 64].contains(v) { maxPlayers = v }
        if let v = json["texture_quality"] as? Int, (0...2).contains(v) { textures = v }
        if let v = json["touch_sensitivity"] as? Double { touch = max(0.1, min(10.0, v)) }
        if let v = json["aim_sensitivity"] as? Double { aimSensitivity = max(0.1, min(5.0, v)) }
        if let v = json["aim_mode"] as? String, ["toggle", "hold"].contains(v) { aimMode = v }
        if let v = json["fire_aim"] as? Bool { fireAim = v }
        if let v = json["mute"] as? Bool { mute = v }
        if let v = json["multiplayer"] as? Bool { multiplayer = v }
        if let v = json["host"] as? Bool { host = v }
        if let v = json["server"] as? String, !v.trimmingCharacters(in: .whitespaces).isEmpty { server = v }
        if let v = json["room_digits"] as? String, !v.trimmingCharacters(in: .whitespaces).isEmpty {
            roomDigits = v
        } else if let v = json["room"] as? String {
            let digits = v.filter { $0.isNumber }
            if !digits.isEmpty { roomDigits = String(digits.prefix(4)) }
        }
        if let v = json["join_code"] as? String { joinCode = v }
        if let v = json["room_pass"] as? String { roomPass = v }
        if let v = json["room_mode"] as? String, ["create", "join", "public", "lan"].contains(v) { roomMode = v }
        if let v = json["pvp"] as? Bool { pvp = v }
        if let v = json["promotion_sync"] as? Bool { promotionSync = v }
        if let v = json["deep_optimization"] as? Bool { deepOptimization = v }
        if let v = json["button_scale"] as? Double { buttonScale = max(0.6, min(2.0, v)) }
        if let v = json["button_offset_x"] as? Double { buttonOffsetX = max(-150.0, min(150.0, v)) }
        if let v = json["button_offset_y"] as? Double { buttonOffsetY = max(-150.0, min(150.0, v)) }
        if let v = json["button_opacity"] as? Double { buttonOpacity = max(0.1, min(0.9, v)) }
        if let v = json["stick_offset_x"] as? Double { stickOffsetX = max(-100.0, min(150.0, v)) }
        if let v = json["stick_offset_y"] as? Double { stickOffsetY = max(-100.0, min(150.0, v)) }
        if let v = json["player_name"] as? String, !v.trimmingCharacters(in: .whitespaces).isEmpty && v != "iPhone" && v != "iPad" {
            name = v
        }
        if let dict = json["custom_buttons"] as? [String: [Double]] {
            for (k, v) in dict {
                if let idx = Int(k), v.count >= 3 {
                    customButtons[idx] = v
                }
            }
        }
        applyCustomLayout()
    }

    func parseJoinCode(_ code: String) {
        let clean = code.trimmingCharacters(in: .whitespaces).uppercased()
        if clean.hasPrefix("S") {
            mode = 1
        } else if clean.hasPrefix("B") {
            mode = 2
        }
        if clean.contains("X") {
            pvp = true
        } else if clean.contains("P") {
            pvp = false
        }
    }

    func save() throws {
        var json = extra
        let srv = server.trimmingCharacters(in: .whitespaces)
        json["server"] = srv.isEmpty ? "18.143.103.253" : srv
        json["multiplayer"] = multiplayer
        json["host"] = (roomMode == "lan")
        json["room_mode"] = roomMode
        json["promotion_sync"] = promotionSync
        json["deep_optimization"] = deepOptimization
        json["max_players"] = maxPlayers
        json["room_digits"] = roomDigits
        json["join_code"] = joinCode

        if multiplayer {
            if roomMode == "create" {
                let m = mode == 1 ? "story" : "sandbox"
                let r = pvp ? "pvp" : "peace"
                let rm = "\(m)-\(r)-\(roomDigits)"
                json["room"] = rm
                json["room_pass"] = roomPass.trimmingCharacters(in: .whitespaces)
                json["start_mode"] = mode
                json["pvp"] = pvp
            } else if roomMode == "join" {
                parseJoinCode(joinCode)
                let m = mode == 1 ? "story" : "sandbox"
                let r = pvp ? "pvp" : "peace"
                let digits = joinCode.filter { $0.isNumber }
                let rm = "\(m)-\(r)-\(digits.isEmpty ? "default" : digits)"
                json["room"] = rm
                json["room_pass"] = roomPass.trimmingCharacters(in: .whitespaces)
                json["start_mode"] = mode
                json["pvp"] = pvp
            } else if roomMode == "public" {
                let m = mode == 1 ? "story" : "sandbox"
                json["room"] = "\(m)-public"
                json["room_pass"] = ""
                json["start_mode"] = mode
                json["pvp"] = false
            } else { // lan
                let m = mode == 1 ? "story" : "sandbox"
                json["room"] = "\(m)-lan"
                json["room_pass"] = ""
                json["start_mode"] = mode
                json["pvp"] = pvp
            }
        } else {
            json["start_mode"] = mode
            json["room"] = "default"
            json["room_pass"] = ""
            json["pvp"] = false
        }

        json["scale"] = scale
        json["fps_limit"] = fps
        json["texture_quality"] = textures
        json["touch_sensitivity"] = (touch * 100).rounded() / 100
        json["aim_sensitivity"] = (aimSensitivity * 100).rounded() / 100
        json["aim_mode"] = aimMode
        json["fire_aim"] = fireAim
        json["mute"] = mute
        json["button_scale"] = (buttonScale * 100).rounded() / 100
        json["button_offset_x"] = (buttonOffsetX * 10).rounded() / 10
        json["button_offset_y"] = (buttonOffsetY * 10).rounded() / 10
        json["button_opacity"] = (buttonOpacity * 100).rounded() / 100
        json["stick_offset_x"] = (stickOffsetX * 10).rounded() / 10
        json["stick_offset_y"] = (stickOffsetY * 10).rounded() / 10
        let nm = name.trimmingCharacters(in: .whitespaces)
        json["player_name"] = nm.isEmpty ? "Player" : nm
        var cbJson: [String: [Double]] = [:]
        for (idx, v) in customButtons {
            cbJson[String(idx)] = v
        }
        json["custom_buttons"] = cbJson
        if json["extra_args"] == nil { json["extra_args"] = [String]() }
        let data = try JSONSerialization.data(withJSONObject: json, options: [.prettyPrinted, .sortedKeys])
        try data.write(to: configURL)
    }

    func resetLayout() {
        buttonScale = 1.0
        buttonOffsetX = 0.0
        buttonOffsetY = 0.0
        buttonOpacity = 0.22
        stickOffsetX = 0.0
        stickOffsetY = 0.0
        customButtons.removeAll()
        muguet_reset_button_layout()
        muguet_apply_button_layout()
    }

    func applyCustomLayout() {
        muguet_reset_button_layout()
        for (idx, v) in customButtons {
            if v.count >= 3 {
                muguet_set_button_layout(UInt32(idx), v[0], v[1], v[2])
            }
        }
        muguet_apply_button_layout()
    }
}

@_silgen_name("muguet_import")
func muguet_import(_ zip: UnsafePointer<CChar>, _ progress: @convention(c) (UInt64, UInt64) -> Void) -> Bool
@_silgen_name("muguet_import_error")
func muguet_import_error() -> UnsafePointer<CChar>?
@_silgen_name("muguet_game_ready")
func muguet_game_ready() -> Bool
@_silgen_name("muguet_has_game_code")
func muguet_has_game_code() -> Bool
@_silgen_name("muguet_reload_settings")
func muguet_reload_settings()
@_silgen_name("muguet_apply_cheat")
func muguet_apply_cheat(_ code: UnsafePointer<CChar>)
@_silgen_name("muguet_is_multiplayer")
func muguet_is_multiplayer() -> Bool
@_silgen_name("muguet_toggle_first_person")
func muguet_toggle_first_person()
@_silgen_name("muguet_set_button_layout")
func muguet_set_button_layout(_ idx: UInt32, _ dx: Double, _ dy: Double, _ scale: Double)
@_silgen_name("muguet_reset_button_layout")
func muguet_reset_button_layout()
@_silgen_name("muguet_apply_button_layout")
func muguet_apply_button_layout()

final class Importer: ObservableObject {
    static let shared = Importer()
    @Published var ready = muguet_game_ready()
    @Published var progress: Double?
    @Published var error: String?
    private var lastProgressTime = 0.0

    func run(_ url: URL) {
        error = nil
        progress = 0
        DispatchQueue.global(qos: .userInitiated).async {
            let scoped = url.startAccessingSecurityScopedResource()
            defer { if scoped { url.stopAccessingSecurityScopedResource() } }
            // File providers must finish materializing the selected ZIP before
            // Rust seeks to its central directory at the end of the archive.
            var coordinationError: NSError?
            var ok = false
            NSFileCoordinator(filePresenter: nil).coordinate(readingItemAt: url, options: [], error: &coordinationError) { localURL in
                ok = localURL.path.withCString { muguet_import($0) { done, total in
                    let now = ProcessInfo.processInfo.systemUptime
                    guard done == total || now - Importer.shared.lastProgressTime >= 0.05 else { return }
                    Importer.shared.lastProgressTime = now
                    let p = total > 0 ? Double(done) / Double(total) : 0
                    DispatchQueue.main.async { Importer.shared.progress = p }
                } }
            }
            let message = ok ? nil : coordinationError?.localizedDescription
                ?? muguet_import_error().map { String(cString: $0) } ?? "unknown error"
            DispatchQueue.main.async {
                self.progress = nil
                self.ready = muguet_game_ready()
                if let message { self.error = "Import failed: \(message)" }
            }
        }
    }
}

struct CheatItem: Identifiable {
    let id = UUID()
    let code: String
    let name: String
    let desc: String
    let category: String
}

let CHEAT_LIST: [CheatItem] = [
    CheatItem(code: "PAINKILLER", name: "Bất tử 5 phút", desc: "Không nhận sát thương trong 5 phút", category: "Nhân vật"),
    CheatItem(code: "TURTLE", name: "Đầy Máu & Giáp", desc: "Hồi phục 100% máu, giáp và sửa xe", category: "Nhân vật"),
    CheatItem(code: "TOOLUP", name: "Tất cả Vũ khí", desc: "Nhận full tất cả vũ khí và đạn", category: "Nhân vật"),
    CheatItem(code: "LAWYERUP", name: "Xóa sao cảnh sát", desc: "Giảm mức độ truy nã về 0", category: "Nhân vật"),
    CheatItem(code: "FUGITIVE", name: "Tăng 1 sao cảnh sát", desc: "Tăng thêm 1 cấp độ truy nã", category: "Nhân vật"),
    CheatItem(code: "POWERUP", name: "Đầy kỹ năng", desc: "Nạp đầy thanh kỹ năng nhân vật", category: "Nhân vật"),
    CheatItem(code: "CATCHME", name: "Chạy siêu nhanh", desc: "Tăng tốc độ di chuyển tối đa", category: "Nhân vật"),
    CheatItem(code: "GOTGILLS", name: "Bơi siêu nhanh", desc: "Tăng tốc độ bơi dưới nước", category: "Nhân vật"),
    CheatItem(code: "HOPTOIT", name: "Siêu nhảy", desc: "Nhảy cao gấp 10 lần bình thường", category: "Nhân vật"),
    CheatItem(code: "SKYDIVE", name: "Trang bị Dù nhảy", desc: "Nhận dù để nhảy từ trên cao", category: "Nhân vật"),
    CheatItem(code: "LIQUOR", name: "Chế độ say rượu", desc: "Đi đứng lảo đảo hiệu ứng say xỉn", category: "Nhân vật"),

    CheatItem(code: "BUZZOFF", name: "Trực thăng Buzzard", desc: "Trực thăng chiến đấu có súng & tên lửa", category: "Phương tiện"),
    CheatItem(code: "COMET", name: "Xe thể thao Comet", desc: "Xe Porsche 2 cửa siêu tốc", category: "Phương tiện"),
    CheatItem(code: "RAPIDGT", name: "Xe đua Rapid GT", desc: "Xe thể thao cao cấp", category: "Phương tiện"),
    CheatItem(code: "ROCKET", name: "Mô tô đua PCJ-600", desc: "Xe máy phân khối lớn tốc độ cao", category: "Phương tiện"),
    CheatItem(code: "OFFROAD", name: "Xe cào cào Sanchez", desc: "Mô tô địa hình vượt đồi núi", category: "Phương tiện"),
    CheatItem(code: "BANDIT", name: "Xe đạp BMX", desc: "Xe đạp biểu diễn nhào lộn", category: "Phương tiện"),
    CheatItem(code: "BARNSTORM", name: "Máy bay Stunt Plane", desc: "Máy bay nhào lộn cánh quạt", category: "Phương tiện"),
    CheatItem(code: "VINEWOOD", name: "Xe Limousine", desc: "Xe sang trọng kéo dài", category: "Phương tiện"),
    CheatItem(code: "TRASHED", name: "Xe rác Trashmaster", desc: "Xe chở rác bọc thép húc xe khác", category: "Phương tiện"),
    CheatItem(code: "DEATHCAR", name: "Xe bọc thép Duke O'Death", desc: "Xe cơ bắp chống đạn chống nổ cực mạnh", category: "Phương tiện"),
    CheatItem(code: "BUBBLES", name: "Tàu ngầm Kraken", desc: "Tàu lặn khám phá đáy đại dương", category: "Phương tiện"),
    CheatItem(code: "FLYSPRAY", name: "Máy bay Duster", desc: "Máy bay cánh quạt cổ điển", category: "Phương tiện"),

    CheatItem(code: "HOTHANDS", name: "Nắm đấm phát nổ", desc: "Đấm kẻ địch văng xa và phát nổ", category: "Chiến đấu"),
    CheatItem(code: "INCENDIARY", name: "Đạn lửa", desc: "Bắn ra đạn thiêu đốt mục tiêu", category: "Chiến đấu"),
    CheatItem(code: "HIGHEX", name: "Đạn nổ", desc: "Mỗi phát bắn tạo ra vụ nổ lớn", category: "Chiến đấu"),
    CheatItem(code: "DEADEYE", name: "Làm chậm khi ngắm", desc: "Làm chậm thời gian khi giữ nút ngắm", category: "Chiến đấu"),

    CheatItem(code: "SKYFALL", name: "Rơi tự do từ trời", desc: "Thả nhân vật từ trên chín tầng mây", category: "Thế giới"),
    CheatItem(code: "SLOWMO", name: "Làm chậm toàn game", desc: "Hiệu ứng slow-motion điện ảnh", category: "Thế giới"),
    CheatItem(code: "FLOATER", name: "Trọng lực Mặt Trăng", desc: "Xe và người bay lơ lửng nhẹ", category: "Thế giới"),
    CheatItem(code: "MAKEITRAIN", name: "Đổi thời tiết", desc: "Thay đổi nắng, mưa, bão sấm chớp", category: "Thế giới"),
    CheatItem(code: "SNOW", name: "Tuyết rơi", desc: "Kích hoạt hiệu ứng tuyết rơi toàn map", category: "Thế giới")
]

struct CheatMenuView: View {
    @Environment(\.dismiss) private var dismiss
    @State private var customCode = ""
    @State private var toastMessage: String?
    let onApply: (String) -> Void

    var body: some View {
        NavigationView {
            List {
                Section(header: Text("Nhập lệnh tùy chỉnh")) {
                    HStack {
                        TextField("Nhập mã lệnh (vd: PAINKILLER)", text: $customCode)
                            .textInputAutocapitalization(.characters)
                            .autocorrectionDisabled()
                        Button("Áp dụng") {
                            let c = customCode.trimmingCharacters(in: .whitespaces).uppercased()
                            guard !c.isEmpty else { return }
                            apply(c)
                        }
                        .buttonStyle(.borderedProminent)
                    }
                }

                ForEach(["Nhân vật", "Phương tiện", "Chiến đấu", "Thế giới"], id: \.self) { cat in
                    Section(header: Text(cat)) {
                        ForEach(CHEAT_LIST.filter { $0.category == cat }) { item in
                            HStack {
                                VStack(alignment: .leading, spacing: 3) {
                                    HStack {
                                        Text(item.name).fontWeight(.semibold)
                                        Text("(\(item.code))").font(.caption.monospaced()).foregroundStyle(.secondary)
                                    }
                                    Text(item.desc).font(.caption2).foregroundStyle(.secondary)
                                }
                                Spacer()
                                Button(action: { apply(item.code) }) {
                                    Text("Bật").font(.caption.weight(.bold)).padding(.horizontal, 10).padding(.vertical, 4)
                                }
                                .buttonStyle(.borderedProminent)
                                .tint(.blue)
                            }
                        }
                    }
                }
            }
            .navigationTitle("Lệnh Cheat GTA V")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Đóng") { dismiss() }
                }
            }
            .overlay(alignment: .bottom) {
                if let msg = toastMessage {
                    Text(msg)
                        .font(.callout.bold())
                        .padding(.horizontal, 16)
                        .padding(.vertical, 10)
                        .background(.ultraThinMaterial)
                        .clipShape(Capsule())
                        .shadow(radius: 5)
                        .padding(.bottom, 20)
                        .transition(.move(edge: .bottom).combined(with: .opacity))
                }
            }
        }
        .navigationViewStyle(StackNavigationViewStyle())
    }

    private func apply(_ code: String) {
        onApply(code)
        withAnimation { toastMessage = "Đã gửi lệnh cheat: \(code)" }
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.2) {
            dismiss()
        }
    }
}

struct HUDButtonDef: Identifiable {
    let id: Int
    let name: String
    let label: String
    let anchorType: AnchorType
    let baseOffset: CGPoint
    let baseRadius: CGFloat
}

enum AnchorType {
    case topLeft
    case topRight
    case bottomRight
    case stick
}

let HUD_BUTTON_DEFS: [HUDButtonDef] = [
    HUDButtonDef(id: 0, name: "FIRE (Bắn / Ga)", label: "FIRE", anchorType: .bottomRight, baseOffset: CGPoint(x: 62, y: 68), baseRadius: 44),
    HUDButtonDef(id: 1, name: "AIM (Ngắm / Phanh)", label: "AIM", anchorType: .bottomRight, baseOffset: CGPoint(x: 158, y: 46), baseRadius: 31),
    HUDButtonDef(id: 2, name: "JUMP (Nhảy / Phanh tay)", label: "JUMP", anchorType: .bottomRight, baseOffset: CGPoint(x: 58, y: 168), baseRadius: 28),
    HUDButtonDef(id: 3, name: "ENTER (Lên/xuống xe)", label: "ENTER", anchorType: .bottomRight, baseOffset: CGPoint(x: 140, y: 132), baseRadius: 26),
    HUDButtonDef(id: 4, name: "RELOAD (Nạp đạn / Còi)", label: "RELOAD", anchorType: .bottomRight, baseOffset: CGPoint(x: 222, y: 98), baseRadius: 24),
    HUDButtonDef(id: 5, name: "WEAPON (Vũ khí)", label: "WEAPON", anchorType: .topRight, baseOffset: CGPoint(x: 30, y: 30), baseRadius: 22),
    HUDButtonDef(id: 6, name: "COVER (Núp / Radio)", label: "COVER", anchorType: .topRight, baseOffset: CGPoint(x: 84, y: 30), baseRadius: 22),
    HUDButtonDef(id: 7, name: "CROUCH (Ngồi / Nhìn sau)", label: "CROUCH", anchorType: .topRight, baseOffset: CGPoint(x: 138, y: 30), baseRadius: 22),
    HUDButtonDef(id: 8, name: "CAM (Góc camera)", label: "CAM", anchorType: .topRight, baseOffset: CGPoint(x: 192, y: 30), baseRadius: 22),
    HUDButtonDef(id: 9, name: "MENU (Muguet Menu)", label: "MENU", anchorType: .topRight, baseOffset: CGPoint(x: 246, y: 30), baseRadius: 22),
    HUDButtonDef(id: 10, name: "PAUSE (Tạm dừng)", label: "PAUSE", anchorType: .topLeft, baseOffset: CGPoint(x: 30, y: 30), baseRadius: 22),
    HUDButtonDef(id: 11, name: "PHONE (Điện thoại)", label: "PHONE", anchorType: .topLeft, baseOffset: CGPoint(x: 84, y: 30), baseRadius: 22),
    HUDButtonDef(id: 12, name: "MENU KEYS (Phím mũi tên)", label: "MENU\nKEYS", anchorType: .topLeft, baseOffset: CGPoint(x: 138, y: 30), baseRadius: 22),
    HUDButtonDef(id: 13, name: "SWITCH (Đổi nhân vật)", label: "SWITCH", anchorType: .topLeft, baseOffset: CGPoint(x: 192, y: 30), baseRadius: 22),
    HUDButtonDef(id: 14, name: "1ST POV (Góc nhìn thứ 1)", label: "1ST\nPOV", anchorType: .topLeft, baseOffset: CGPoint(x: 246, y: 30), baseRadius: 22),
    HUDButtonDef(id: 15, name: "FIRE phụ (Bắn trái)", label: "FIRE", anchorType: .topLeft, baseOffset: CGPoint(x: 45, y: 112), baseRadius: 29),
    HUDButtonDef(id: 16, name: "JOYSTICK (Cần di chuyển)", label: "", anchorType: .stick, baseOffset: CGPoint(x: 0, y: 0), baseRadius: 58)
]

struct HUDEditorView: View {
    @Environment(\.dismiss) private var dismiss
    @ObservedObject var settings: Settings
    @State private var selectedId: Int = 0
    @State private var offsets: [Int: CGSize] = [:]
    @State private var scales: [Int: Double] = [:]
    @State private var draggingId: Int? = nil
    @State private var dragTranslation: CGSize = .zero
    @State private var toastMessage: String?
    @State private var isCollapsed: Bool = false
    @State private var manualDockTop: Bool? = nil

    var body: some View {
        GeometryReader { geo in
            ZStack {
                Color.black.ignoresSafeArea()

                // Subtle grid background
                Path { path in
                    let step: CGFloat = 40
                    for x in stride(from: 0, through: geo.size.width, by: step) {
                        path.move(to: CGPoint(x: x, y: 0))
                        path.addLine(to: CGPoint(x: x, y: geo.size.height))
                    }
                    for y in stride(from: 0, through: geo.size.height, by: step) {
                        path.move(to: CGPoint(x: 0, y: y))
                        path.addLine(to: CGPoint(x: geo.size.width, y: y))
                    }
                }
                .stroke(Color.white.opacity(0.04), lineWidth: 1)
                .ignoresSafeArea()

                // Screen center crosshair
                Path { path in
                    let cx = geo.size.width / 2
                    let cy = geo.size.height / 2
                    path.move(to: CGPoint(x: cx - 20, y: cy))
                    path.addLine(to: CGPoint(x: cx + 20, y: cy))
                    path.move(to: CGPoint(x: cx, y: cy - 20))
                    path.addLine(to: CGPoint(x: cx, y: cy + 20))
                }
                .stroke(Color.white.opacity(0.15), lineWidth: 1)

                // Render Buttons Canvas
                ForEach(HUD_BUTTON_DEFS) { def in
                    renderButton(def, in: geo)
                }

                // Floating Dynamic Toolbar (never obscures selected button)
                VStack(spacing: 6) {
                    if isCollapsed {
                        if shouldDockTop(in: geo) {
                            collapsedBar(geo: geo, isTop: true)
                        } else {
                            collapsedBar(geo: geo, isTop: false)
                        }
                    } else {
                        let isTop = shouldDockTop(in: geo)
                        if isTop {
                            editorToolbar(geo: geo, isTop: true)
                            selectedButtonControls
                        } else {
                            selectedButtonControls
                            editorToolbar(geo: geo, isTop: false)
                        }
                    }
                }
                .padding(.horizontal, 16)
                .padding(.vertical, 8)
                // Disable only the transparent background; disabling this parent
                // also disables its toolbar buttons and sliders.
                .background(Color.clear.allowsHitTesting(false))
                .frame(maxWidth: .infinity, maxHeight: .infinity,
                       alignment: shouldDockTop(in: geo) ? .top : .bottom)
                .animation(.easeInOut(duration: 0.25), value: isCollapsed)
                .animation(.easeInOut(duration: 0.25), value: manualDockTop)
                .animation(.easeInOut(duration: 0.25), value: selectedId)

                // Toast notification
                if let msg = toastMessage {
                    VStack {
                        Spacer()
                        Text(msg)
                            .font(.callout.bold())
                            .foregroundColor(.white)
                            .padding(.horizontal, 16)
                            .padding(.vertical, 10)
                            .background(Color.black.opacity(0.85))
                            .clipShape(Capsule())
                            .overlay(Capsule().stroke(Color.blue, lineWidth: 1))
                            .shadow(radius: 6)
                            .padding(.bottom, 24)
                            .transition(.move(edge: .bottom).combined(with: .opacity))
                    }
                    .allowsHitTesting(false)
                }
            }
            .coordinateSpace(name: "hudCanvas")
        }
        .onAppear(perform: loadLayout)
    }

    private func shouldDockTop(in geo: GeometryProxy) -> Bool {
        if let manual = manualDockTop { return manual }
        return false
    }

    private func editorToolbar(geo: GeometryProxy, isTop: Bool) -> some View {
        HStack(spacing: 10) {
            Button("Hủy") {
                dismiss()
            }
            .buttonStyle(.bordered)
            .tint(.secondary)

            Button(action: {
                let current = shouldDockTop(in: geo)
                manualDockTop = !current
            }) {
                HStack(spacing: 4) {
                    Image(systemName: isTop ? "arrow.down.to.line" : "arrow.up.to.line")
                    Text(isTop ? "Xuống dưới" : "Lên trên")
                }
                .font(.caption.bold())
            }
            .buttonStyle(.bordered)
            .tint(.cyan)

            Spacer()

            Text("HUD EDITOR")
                .font(.subheadline.bold())
                .foregroundColor(.white)

            Spacer()

            Button(action: {
                isCollapsed = true
            }) {
                HStack(spacing: 4) {
                    Image(systemName: "arrow.down.right.and.arrow.up.left")
                    Text("Thu gọn")
                }
                .font(.caption.bold())
            }
            .buttonStyle(.bordered)
            .tint(.orange)

            Button("Đặt lại tất cả") {
                for i in 0..<17 {
                    offsets[i] = .zero
                    scales[i] = 1.0
                }
                showToast("Đã đưa toàn bộ nút về mặc định")
            }
            .buttonStyle(.bordered)
            .tint(.red)

            Button("Lưu & Áp dụng") {
                saveLayout()
                dismiss()
            }
            .buttonStyle(.borderedProminent)
            .tint(.green)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 12))
        .allowsHitTesting(true)
    }

    private func collapsedBar(geo: GeometryProxy, isTop: Bool) -> some View {
        HStack(spacing: 12) {
            Button("Hủy") { dismiss() }
                .buttonStyle(.bordered)
                .tint(.secondary)

            Spacer()

            Button(action: { isCollapsed = false }) {
                HStack(spacing: 6) {
                    Image(systemName: "slider.horizontal.3")
                    Text("Hiện bảng HUD Editor")
                }
                .font(.caption.bold())
                .padding(.horizontal, 8)
                .padding(.vertical, 4)
            }
            .buttonStyle(.borderedProminent)
            .tint(.blue)

            Spacer()

            Button("Lưu & Áp dụng") {
                saveLayout()
                dismiss()
            }
            .buttonStyle(.borderedProminent)
            .tint(.green)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(.ultraThinMaterial)
        .clipShape(Capsule())
        .shadow(radius: 6)
        .allowsHitTesting(true)
    }

    private var selectedButtonControls: some View {
        let currentDef = HUD_BUTTON_DEFS.first(where: { $0.id == selectedId }) ?? HUD_BUTTON_DEFS[0]
        let currentScale = scales[selectedId] ?? 1.0

        return HStack(spacing: 14) {
            // Button selector picker
            Menu {
                ForEach(HUD_BUTTON_DEFS) { def in
                    Button(def.name) {
                        selectedId = def.id
                    }
                }
            } label: {
                HStack(spacing: 4) {
                    Text(currentDef.name)
                        .font(.caption.bold())
                        .foregroundColor(.cyan)
                    Image(systemName: "chevron.up.chevron.down")
                        .font(.caption2)
                        .foregroundColor(.cyan)
                }
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
                .background(Color.cyan.opacity(0.15))
                .clipShape(RoundedRectangle(cornerRadius: 8))
            }

            Divider().frame(height: 20)

            // Scale slider & buttons
            HStack(spacing: 6) {
                Text("Cỡ:")
                    .font(.caption)
                    .foregroundColor(.secondary)

                Button(action: { adjustScale(-0.05) }) {
                    Image(systemName: "minus.circle")
                        .font(.caption.bold())
                }

                Slider(value: Binding(
                    get: { scales[selectedId] ?? 1.0 },
                    set: { scales[selectedId] = (round($0 * 20) / 20) }
                ), in: 0.5...2.2, step: 0.05)
                .frame(width: 110)

                Button(action: { adjustScale(0.05) }) {
                    Image(systemName: "plus.circle")
                        .font(.caption.bold())
                }

                Text(String(format: "%.2f×", currentScale))
                    .font(.caption.monospacedDigit())
                    .frame(width: 42, alignment: .trailing)
            }

            Divider().frame(height: 20)

            // D-Pad nudge arrows
            HStack(spacing: 3) {
                Text("Dời:")
                    .font(.caption)
                    .foregroundColor(.secondary)
                Button(action: { nudge(dx: -4, dy: 0) }) {
                    Image(systemName: "arrow.left.square.fill")
                }
                Button(action: { nudge(dx: 0, dy: -4) }) {
                    Image(systemName: "arrow.up.square.fill")
                }
                Button(action: { nudge(dx: 0, dy: 4) }) {
                    Image(systemName: "arrow.down.square.fill")
                }
                Button(action: { nudge(dx: 4, dy: 0) }) {
                    Image(systemName: "arrow.right.square.fill")
                }
            }
            .font(.body)

            Divider().frame(height: 20)

            Button("Đặt lại nút") {
                offsets[selectedId] = .zero
                scales[selectedId] = 1.0
                showToast("Đã đặt lại \(currentDef.name)")
            }
            .font(.caption)
            .buttonStyle(.bordered)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 10))
        .allowsHitTesting(true)
    }

    private func renderButton(_ def: HUDButtonDef, in geo: GeometryProxy) -> some View {
        let isSelected = (selectedId == def.id)
        let pos = buttonCenter(for: def, in: geo)
        let bScale = CGFloat(settings.buttonScale)
        let cScale = CGFloat(scales[def.id] ?? 1.0)
        let radius = def.baseRadius * bScale * cScale
        let diameter = radius * 2.0

        return ZStack {
            if def.id == 16 {
                Circle()
                    .stroke(isSelected ? Color.yellow : Color.white.opacity(0.35), lineWidth: isSelected ? 3 : 1.5)
                    .background(Circle().fill(Color.white.opacity(0.12)))
                    .frame(width: diameter, height: diameter)
                Circle()
                    .fill(isSelected ? Color.yellow.opacity(0.7) : Color.white.opacity(0.35))
                    .frame(width: diameter * (26.0 / 58.0), height: diameter * (26.0 / 58.0))
            } else {
                Circle()
                    .stroke(isSelected ? Color.yellow : Color.white.opacity(0.4), lineWidth: isSelected ? 3 : 1.5)
                    .background(Circle().fill(isSelected ? Color.white.opacity(0.45) : Color.white.opacity(settings.buttonOpacity)))
                    .frame(width: diameter, height: diameter)
                    .shadow(color: isSelected ? Color.yellow.opacity(0.8) : Color.black.opacity(0.3), radius: isSelected ? 6 : 2)

                Text(def.label)
                    .font(.system(size: max(8, min(16, radius * 0.42)), weight: .bold))
                    .foregroundColor(Color.white.opacity(0.95))
                    .multilineTextAlignment(.center)
            }
        }
        // Keep the hit region the size of the button, before positioning it.
        .frame(width: diameter, height: diameter)
        .contentShape(Circle().scale(1.3))
        .gesture(
            DragGesture(minimumDistance: 0, coordinateSpace: .named("hudCanvas"))
                .onChanged { value in
                    if selectedId != def.id {
                        selectedId = def.id
                    }
                    draggingId = def.id
                    dragTranslation = value.translation
                }
                .onEnded { value in
                    let base = offsets[def.id] ?? .zero
                    offsets[def.id] = CGSize(
                        width: base.width + value.translation.width,
                        height: base.height + value.translation.height
                    )
                    dragTranslation = .zero
                    draggingId = nil
                }
        )
        .position(x: pos.x, y: pos.y)
    }

    private func buttonCenter(for def: HUDButtonDef, in geo: GeometryProxy) -> CGPoint {
        let sw = geo.size.width
        let sh = geo.size.height

        let bScale = CGFloat(settings.buttonScale)
        let bOx = CGFloat(settings.buttonOffsetX)
        let bOy = CGFloat(settings.buttonOffsetY)

        let saved = offsets[def.id] ?? .zero
        let currentDrag = (draggingId == def.id) ? dragTranslation : .zero
        let cDx = saved.width + currentDrag.width
        let cDy = saved.height + currentDrag.height
        let cScale = CGFloat(scales[def.id] ?? 1.0)

        var x: CGFloat = 0
        var y: CGFloat = 0

        switch def.anchorType {
        case .bottomRight:
            x = sw - def.baseOffset.x * bScale - bOx + cDx
            y = sh - def.baseOffset.y * bScale - bOy + cDy
        case .topRight:
            x = sw - def.baseOffset.x * bScale + cDx
            y = def.baseOffset.y * bScale + cDy
        case .topLeft:
            x = def.baseOffset.x * bScale + cDx
            y = def.baseOffset.y * bScale + cDy
        case .stick:
            let sOx = CGFloat(settings.stickOffsetX)
            let sOy = CGFloat(settings.stickOffsetY)
            let stickRadius = 58.0 * bScale * cScale
            x = 40.0 + stickRadius + sOx + cDx
            y = sh - 30.0 - stickRadius - sOy + cDy
        }

        return CGPoint(x: x, y: y)
    }

    private func loadLayout() {
        for i in 0..<17 {
            if let arr = settings.customButtons[i], arr.count >= 3 {
                offsets[i] = CGSize(width: arr[0], height: arr[1])
                scales[i] = arr[2]
            } else {
                offsets[i] = .zero
                scales[i] = 1.0
            }
        }
    }

    private func saveLayout() {
        for i in 0..<17 {
            let off = offsets[i] ?? .zero
            let sc = scales[i] ?? 1.0
            if abs(off.width) < 0.5 && abs(off.height) < 0.5 && abs(sc - 1.0) < 0.01 {
                settings.customButtons.removeValue(forKey: i)
            } else {
                settings.customButtons[i] = [Double(off.width), Double(off.height), sc]
            }
        }
        do { try settings.save() } catch {}
        settings.applyCustomLayout()
    }

    private func adjustScale(_ delta: Double) {
        let current = scales[selectedId] ?? 1.0
        let next = max(0.5, min(2.2, (round((current + delta) * 20) / 20)))
        scales[selectedId] = next
    }

    private func nudge(dx: CGFloat, dy: CGFloat) {
        let current = offsets[selectedId] ?? .zero
        offsets[selectedId] = CGSize(width: current.width + dx, height: current.height + dy)
    }

    private func showToast(_ msg: String) {
        withAnimation { toastMessage = msg }
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.2) {
            withAnimation { toastMessage = nil }
        }
    }
}

struct LauncherView: View {
    @StateObject private var s = Settings()
    @ObservedObject private var importer = Importer.shared
    @State private var error: String?
    @State private var picking = false
    @State private var showingCheatSheet = false
    @State private var showingHUDEditor = false
    var isOverlay: Bool = false
    let play: () -> Void

    var body: some View {
        HStack(spacing: 0) {
            VStack(spacing: 16) {
                Text(isOverlay ? "Muguet Menu" : "Muguet").font(.largeTitle.weight(.bold))
                if !muguet_has_game_code() {
                    Text("This build has no game code: build it with ./make-ios.sh <game.zip>.")
                        .foregroundStyle(.red).font(.callout).multilineTextAlignment(.center)
                } else if !importer.ready || importer.progress != nil {
                    importView
                } else if isOverlay {
                    Button(action: resumeAndSave) {
                        Text("Tiếp tục (Resume)").font(.title3.weight(.semibold)).frame(maxWidth: 220)
                    }
                    .controlSize(.large)
                    .buttonStyle(.borderedProminent)
                    .tint(.green)

                    Button(action: { exit(0) }) {
                        Text("Thoát hẳn ứng dụng (Exit)").font(.callout.weight(.medium)).frame(maxWidth: 220)
                    }
                    .controlSize(.regular)
                    .buttonStyle(.bordered)
                    .tint(.red)
                } else {
                    if let error { Text(error).foregroundStyle(.red).font(.callout).multilineTextAlignment(.center) }
                    playButton
                    Button("Import other game data…") { picking = true }.font(.caption)
                }
                Text("proudly made by c22dev").font(.caption).foregroundStyle(.secondary)
                Text("Rebuild: iOSVN").font(.caption).foregroundStyle(.secondary)
            }
            .padding(24)
            .frame(maxWidth: .infinity)
            Form {
                Section("Tên nhân vật / Người chơi") {
                    HStack {
                        Image(systemName: "person.crop.circle.fill")
                            .foregroundStyle(.blue)
                        TextField("Nhập tên nhân vật...", text: $s.name)
                            .textInputAutocapitalization(.words)
                            .autocorrectionDisabled()
                    }
                    Text("Tên này sẽ hiển thị trên đầu nhân vật của bạn khi chơi cùng bạn bè.")
                        .font(.caption).foregroundStyle(.secondary)
                }

                if !s.multiplayer {
                    Picker("Mode", selection: $s.mode) {
                        Text("Story Mode (Cốt truyện)").tag(1)
                        Text("Sandbox (Tự do)").tag(2)
                    }
                }

                Section("Cấu hình Đồ họa & Kích hoạt 120 FPS") {
                    Button(action: {
                        s.apply120FpsBoost()
                    }) {
                        HStack(spacing: 8) {
                            Image(systemName: "bolt.fill").foregroundStyle(.yellow)
                            VStack(alignment: .leading, spacing: 2) {
                                Text("⚡ Kích hoạt Cấu hình 120 FPS Siêu Mượt").bold()
                                Text("Tự động đặt 120 FPS, 50% Resolution, Textures Normal, bật ProMotion V-Sync & Deep Optimization.")
                                    .font(.caption2).foregroundStyle(.secondary)
                            }
                        }
                    }
                    .buttonStyle(.borderedProminent)
                    .tint(.indigo)

                    Picker("Tốc độ khung hình (Frame Limit)", selection: $s.fps) {
                        Text("30 fps (Tiết kiệm pin)").tag(30)
                        Text("60 fps (Mặc định)").tag(60)
                        Text("120 fps (Mở khóa 120 FPS / ProMotion)").tag(120)
                    }

                    Picker("Độ phân giải (Resolution)", selection: $s.scale) {
                        Text("35 % (Siêu nhẹ)").tag(0.35)
                        Text("50 % (Khuyên dùng cho 120 FPS)").tag(0.5)
                        Text("65 % (Cân bằng)").tag(0.65)
                        Text("75 % (Rõ nét)").tag(0.75)
                        Text("100 % (Độ phân giải gốc)").tag(1.0)
                    }

                    Picker("Chất lượng Textures", selection: $s.textures) {
                        Text("Bình thường / Thấp (Ít tốn RAM)").tag(0)
                        Text("Cao").tag(1)
                        Text("Rất cao").tag(2)
                    }

                    Toggle("Khóa nhịp ProMotion V-Sync 120Hz (CADisplayLink)", isOn: $s.promotionSync)
                    Text("Đồng bộ chính xác với tần số quét 60Hz/120Hz của màn hình iPhone/iPad để triệt tiêu khựng hình và giữ khung hình 120 FPS mượt nhất.")
                        .font(.caption).foregroundStyle(.secondary)

                    Toggle("Tối ưu đồ họa sâu (Deep Optimization 60-120 FPS Boost)", isOn: $s.deepOptimization)
                    Text("Tự động khống chế bóng đổ, cỏ rậm và khử răng cưa trong GTA V settings.xml để duy trì FPS tối đa khi máy nóng.")
                        .font(.caption).foregroundStyle(.secondary)
                }

                Section("Tùy chỉnh Giao diện Nút Bấm (Touch Controls)") {
                    Button(action: { showingHUDEditor = true }) {
                        Label("Chỉnh vị trí & Kích thước nút trực quan (HUD Editor)", systemImage: "hand.draw.fill")
                            .font(.subheadline.bold())
                            .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.borderedProminent)
                    .tint(.blue)

                    HStack {
                        Text("Kích thước nút")
                        Slider(value: $s.buttonScale, in: 0.7...1.5, step: 0.05)
                        Text(String(format: "%.2f×", s.buttonScale)).monospacedDigit().frame(width: 55, alignment: .trailing)
                    }
                    HStack {
                        Text("Độ trong suốt nút")
                        Slider(value: $s.buttonOpacity, in: 0.12...0.70, step: 0.02)
                        Text(String(format: "%.0f%%", s.buttonOpacity * 100)).monospacedDigit().frame(width: 55, alignment: .trailing)
                    }
                    HStack {
                        Text("Dời cụm nút (Ngang X)")
                        Slider(value: $s.buttonOffsetX, in: -60.0...60.0, step: 5.0)
                        Text(String(format: "%+.0f", s.buttonOffsetX)).monospacedDigit().frame(width: 55, alignment: .trailing)
                    }
                    HStack {
                        Text("Dời cụm nút (Dọc Y)")
                        Slider(value: $s.buttonOffsetY, in: -60.0...60.0, step: 5.0)
                        Text(String(format: "%+.0f", s.buttonOffsetY)).monospacedDigit().frame(width: 55, alignment: .trailing)
                    }
                    HStack {
                        Text("Dời Joystick (Ngang X)")
                        Slider(value: $s.stickOffsetX, in: -40.0...60.0, step: 5.0)
                        Text(String(format: "%+.0f", s.stickOffsetX)).monospacedDigit().frame(width: 55, alignment: .trailing)
                    }
                    HStack {
                        Text("Dời Joystick (Dọc Y)")
                        Slider(value: $s.stickOffsetY, in: -40.0...60.0, step: 5.0)
                        Text(String(format: "%+.0f", s.stickOffsetY)).monospacedDigit().frame(width: 55, alignment: .trailing)
                    }
                    Button("Khôi phục mặc định nút bấm") {
                        s.resetLayout()
                    }
                    .font(.caption)
                    .foregroundStyle(.orange)
                }

                HStack {
                    Text("Look sensitivity")
                    Slider(value: $s.touch, in: 0.25...6.0, step: 0.25)
                    Text(String(format: "%.2f×", s.touch)).monospacedDigit().frame(width: 55, alignment: .trailing)
                }
                Section("Aiming") {
                    Picker("Aim button", selection: $s.aimMode) {
                        Text("Tap to toggle").tag("toggle")
                        Text("Hold to aim").tag("hold")
                    }
                    Toggle("Aim while firing", isOn: $s.fireAim)
                    HStack {
                        Text("Aim sensitivity")
                        Slider(value: $s.aimSensitivity, in: 0.1...3.0, step: 0.1)
                        Text(String(format: "%.2f×", s.aimSensitivity)).monospacedDigit().frame(width: 55, alignment: .trailing)
                    }
                    Text("Drag on the right FIRE button to adjust aim while shooting. The left FIRE button frees your right thumb to aim.")
                        .font(.caption).foregroundStyle(.secondary)
                }
                Toggle("Mute audio", isOn: $s.mute)

                Section("Chơi Online (Multiplayer)") {
                    Toggle("Bật chế độ Multiplayer", isOn: $s.multiplayer)
                    if s.multiplayer {
                        Picker("Chế độ phòng", selection: $s.roomMode) {
                            Text("Tạo phòng").tag("create")
                            Text("Vào phòng").tag("join")
                            Text("Công khai").tag("public")
                            Text("LAN Host").tag("lan")
                        }
                        .pickerStyle(.segmented)

                        if s.roomMode == "create" {
                            VStack(alignment: .leading, spacing: 12) {
                                VStack(alignment: .leading, spacing: 4) {
                                    Text("1. Chọn Chế độ phòng:").font(.subheadline.bold())
                                    Picker("Mode", selection: $s.mode) {
                                        Text("Story Mode (Cốt truyện)").tag(1)
                                        Text("Sandbox (Tự do)").tag(2)
                                    }
                                    .pickerStyle(.segmented)
                                }

                                VStack(alignment: .leading, spacing: 4) {
                                    Text("2. Luật sát thương (PvP):").font(.subheadline.bold())
                                    Picker("PvP", selection: $s.pvp) {
                                        Text("🛡️ Hòa bình (Co-op)").tag(false)
                                        Text("⚔️ Đấu trường (PvP)").tag(true)
                                    }
                                    .pickerStyle(.segmented)
                                    Text(s.pvp ? "PvP Bật: Người chơi có thể bắn và tiêu diệt nhau." : "Hòa bình: Không gây sát thương lẫn nhau, tránh bắn nhau khi spawn.")
                                        .font(.caption2).foregroundStyle(.secondary)
                                }

                                VStack(alignment: .leading, spacing: 4) {
                                    Text("3. Số người chơi tối đa (Server capacity):").font(.subheadline.bold())
                                    Picker("Giới hạn người chơi", selection: $s.maxPlayers) {
                                        Text("32 Người chơi").tag(32)
                                        Text("64 Người chơi").tag(64)
                                    }
                                    .pickerStyle(.segmented)
                                    Text("Tạo phòng hỗ trợ tối đa \(s.maxPlayers) người chơi trực tuyến cùng lúc.")
                                        .font(.caption2).foregroundStyle(.secondary)
                                }

                                VStack(alignment: .leading, spacing: 4) {
                                    Text("4. Mã phòng chia sẻ cho bạn bè:").font(.subheadline.bold())
                                    HStack(spacing: 8) {
                                        Text(s.formattedCreateCode)
                                            .font(.title2.monospaced().bold())
                                            .foregroundStyle(.green)
                                            .padding(.horizontal, 10)
                                            .padding(.vertical, 6)
                                            .background(Color.green.opacity(0.15))
                                            .clipShape(RoundedRectangle(cornerRadius: 8))
                                            .fixedSize(horizontal: true, vertical: false)
                                            .layoutPriority(2)

                                        Spacer(minLength: 4)

                                        Button(action: {
                                            s.roomDigits = String(Int.random(in: 1000...9999))
                                        }) {
                                            Image(systemName: "arrow.clockwise")
                                                .font(.caption.bold())
                                                .padding(.horizontal, 8)
                                                .padding(.vertical, 7)
                                        }
                                        .buttonStyle(.bordered)
                                        .help("Đổi mã")

                                        Button(action: {
                                            UIPasteboard.general.string = s.formattedCreateCode
                                        }) {
                                            HStack(spacing: 4) {
                                                Image(systemName: "doc.on.doc")
                                                Text("Chép")
                                            }
                                            .font(.caption.bold())
                                            .padding(.horizontal, 10)
                                            .padding(.vertical, 7)
                                        }
                                        .buttonStyle(.borderedProminent)
                                        .fixedSize()
                                    }
                                }

                                HStack {
                                    Image(systemName: "lock.fill").foregroundStyle(.secondary)
                                    TextField("Mật khẩu phòng (Tùy chọn)", text: $s.roomPass)
                                        .textInputAutocapitalization(.never)
                                        .autocorrectionDisabled()
                                }

                                HStack(spacing: 8) {
                                    Label(s.mode == 1 ? "Story Mode" : "Sandbox", systemImage: "gamecontroller")
                                        .font(.caption.bold())
                                        .foregroundStyle(s.mode == 1 ? .orange : .cyan)
                                    Text("•")
                                    Label(s.pvp ? "PvP Bật" : "Hòa bình Co-op", systemImage: s.pvp ? "burst.fill" : "shield.fill")
                                        .font(.caption.bold())
                                        .foregroundStyle(s.pvp ? .red : .green)
                                    Text("•")
                                    Label(s.roomPass.isEmpty ? "Phòng Mở" : "Có Pass", systemImage: s.roomPass.isEmpty ? "lock.open" : "lock")
                                        .font(.caption.bold())
                                        .foregroundStyle(.secondary)
                                }
                                .padding(8)
                                .background(Color.secondary.opacity(0.1))
                                .clipShape(RoundedRectangle(cornerRadius: 8))
                            }
                        } else if s.roomMode == "join" {
                            VStack(alignment: .leading, spacing: 12) {
                                VStack(alignment: .leading, spacing: 4) {
                                    Text("Nhập mã phòng:").font(.subheadline.bold())
                                    TextField("Ví dụ: SP-8621 hoặc BX-5520", text: $s.joinCode)
                                        .font(.title3.monospaced().bold())
                                        .textInputAutocapitalization(.characters)
                                        .autocorrectionDisabled()
                                        .onChange(of: s.joinCode) { val in
                                            s.parseJoinCode(val)
                                        }
                                }

                                let clean = s.joinCode.trimmingCharacters(in: .whitespaces).uppercased()
                                if !clean.isEmpty {
                                    HStack(spacing: 8) {
                                        Label(s.mode == 1 ? "Story Mode" : "Sandbox", systemImage: "gamecontroller")
                                            .font(.caption.bold())
                                            .foregroundStyle(s.mode == 1 ? .orange : .cyan)
                                        Text("•")
                                        Label(s.pvp ? "PvP Bật" : "Hòa bình Co-op", systemImage: s.pvp ? "burst.fill" : "shield.fill")
                                            .font(.caption.bold())
                                            .foregroundStyle(s.pvp ? .red : .green)
                                        Spacer()
                                        Text("Đã đồng bộ tự động")
                                            .font(.caption2)
                                            .foregroundStyle(.secondary)
                                    }
                                    .padding(8)
                                    .background(Color.green.opacity(0.12))
                                    .clipShape(RoundedRectangle(cornerRadius: 8))
                                } else {
                                    Text("Chế độ chơi (Story / Sandbox) và luật PvP tự động đồng bộ theo mã phòng của chủ phòng.")
                                        .font(.caption2).foregroundStyle(.secondary)
                                }

                                HStack {
                                    Image(systemName: "lock.fill").foregroundStyle(.secondary)
                                    TextField("Mật khẩu phòng (nếu chủ phòng đặt)", text: $s.roomPass)
                                        .textInputAutocapitalization(.never)
                                        .autocorrectionDisabled()
                                }

                                Text("Lưu ý: Luật sát thương PvP và Chế độ chơi hoàn toàn tuân theo chủ phòng, không thể tự đổi.")
                                    .font(.caption2).foregroundStyle(.orange)
                            }
                        } else if s.roomMode == "public" {
                            VStack(alignment: .leading, spacing: 10) {
                                Picker("Phân vùng công khai", selection: $s.mode) {
                                    Text("Story Mode Public").tag(1)
                                    Text("Sandbox Public").tag(2)
                                }
                                .pickerStyle(.segmented)

                                HStack {
                                    Image(systemName: "globe.americas.fill").foregroundStyle(.green)
                                    Text("Sảnh chung công khai (Hỗ trợ 32 / 64 người/phòng)")
                                        .fontWeight(.medium)
                                }
                                Text("Tự động kết nối vào sảnh chung với người chơi trực tuyến trên server AWS iOSVN. Chế độ Hòa bình bật mặc định.")
                                    .font(.caption).foregroundStyle(.secondary)
                            }
                        } else {
                            VStack(alignment: .leading, spacing: 8) {
                                Picker("Chế độ LAN", selection: $s.mode) {
                                    Text("Story Mode").tag(1)
                                    Text("Sandbox").tag(2)
                                }
                                .pickerStyle(.segmented)
                                Text("Mở phòng trên Wi-Fi (Tối đa \(s.maxPlayers) người): Kết nối qua: \(localIPs().joined(separator: " hoặc "))")
                                    .font(.callout).textSelection(.enabled)
                            }
                        }

                        DisclosureGroup("Máy chủ (Nâng cao)") {
                            HStack {
                                Text("Server IP:")
                                TextField("18.143.103.253", text: $s.server)
                                    .keyboardType(.numbersAndPunctuation)
                                    .autocorrectionDisabled()
                                    .textInputAutocapitalization(.never)
                            }
                            HStack {
                                Circle().fill(Color.green).frame(width: 8, height: 8)
                                Text("Server AWS iOSVN Singapore (Ping ~40ms, Hỗ trợ 32 / 64 người)")
                                    .font(.caption2).foregroundStyle(.secondary)
                            }
                        }
                    }
                }

                if !s.multiplayer {
                    Section("Lệnh Cheat GTA V (Chỉ Offline)") {
                        Button(action: { showingCheatSheet = true }) {
                            Label("Xem & Kích hoạt Lệnh Cheat", systemImage: "wand.and.stars")
                        }
                        Text("Tra cứu và kích hoạt nhanh các lệnh cheat (Bất tử, full vũ khí, trực thăng, xe đua...). Lệnh cheat tự động bị khóa trong chế độ Online.")
                            .font(.caption).foregroundStyle(.secondary)
                    }
                }

                Section("Save Data") {
                    Button("Share / Backup saves") { shareSaves() }
                    Text("In game: save via Phone > Quick Save, or Pause > Game > Save Game. Saves are stored in Files app > On My iPhone > Muguet > saves.")
                        .font(.caption).foregroundStyle(.secondary)
                }
                Section("Diagnostics") {
                    Button("Share startup / memory logs") { shareDiagnostics() }
                    Text("After a crash, reopen Muguet and share the previous log. Logs contain connection addresses and runtime errors, not game files.")
                        .font(.caption).foregroundStyle(.secondary)
                }
            }
            .frame(maxWidth: .infinity)
        }
        .preferredColorScheme(.dark)
        .fileImporter(isPresented: $picking, allowedContentTypes: [.zip]) { result in
            if case .success(let url) = result { importer.run(url) }
        }
        .sheet(isPresented: $showingCheatSheet) {
            CheatMenuView(onApply: { code in
                code.withCString { muguet_apply_cheat($0) }
            })
        }
        .fullScreenCover(isPresented: $showingHUDEditor) {
            HUDEditorView(settings: s)
        }
    }

    @ViewBuilder var importView: some View {
        VStack(spacing: 12) {
            if let p = importer.progress {
                ProgressView(value: p)
                Text("Importing… \(Int(p * 100)) %").font(.callout).monospacedDigit()
            } else {
                Button("Import game data .zip") { picking = true }
                    .controlSize(.large)
                    .buttonStyle(.borderedProminent)

                VStack(alignment: .leading, spacing: 5) {
                    Text("💡 Hướng dẫn cài đặt nhanh qua ứng dụng Tệp:")
                        .font(.caption.bold())
                        .foregroundStyle(.orange)
                    Text("1. Giải nén file zip chứa data trên máy tính hoặc app Tệp.\n2. Mở app Tệp (Files) > Trên iPhone/iPad > Muguet.\n3. Tạo thư mục tên là 'game' (chữ thường).\n4. Dán 2 thư mục 'data' và 'b' vào bên trong thư mục 'game'.")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                    Divider().padding(.vertical, 2)
                    Text("📦 Lưu ý dung lượng: Game đầy đủ chiếm ~21 GB. Nếu giải nén trực tiếp trên iPhone/iPad cần trống tối thiểu 42 GB (21 GB zip + 21 GB giải nén; sau khi xong xóa file zip đi còn 21 GB). Giải nén trước trên máy tính rồi chép vào sẽ chỉ tốn 21 GB bộ nhớ máy.")
                        .font(.caption2)
                        .foregroundStyle(.yellow)
                }
                .padding(10)
                .background(Color.secondary.opacity(0.12))
                .clipShape(RoundedRectangle(cornerRadius: 8))
            }
            if let e = importer.error { Text(e).foregroundStyle(.red).font(.callout).multilineTextAlignment(.center) }
        }
    }

    @ViewBuilder var playButton: some View {
        let button = Button(action: start) {
            Text("Play").font(.title3.weight(.semibold)).frame(maxWidth: 220)
        }
        .controlSize(.large)
        #if compiler(>=6.2)
        if #available(iOS 26.0, *) {
            button.buttonStyle(.glassProminent)
        } else {
            button.buttonStyle(.borderedProminent)
        }
        #else
        button.buttonStyle(.borderedProminent)
        #endif
    }

    func resumeAndSave() {
        do { try s.save() } catch {}
        s.applyCustomLayout()
        play()
        muguet_reload_settings()
    }

    func start() {
        if s.multiplayer && s.roomMode == "join" {
            let digits = s.joinCode.filter { $0.isNumber }
            if digits.isEmpty {
                error = "Nhập mã phòng (vd: SP-8621) để vào chơi cùng bạn bè."
                return
            }
        }
        if s.multiplayer && s.roomMode != "lan" && s.server.trimmingCharacters(in: .whitespaces).isEmpty {
            error = "Nhập địa chỉ máy chủ để kết nối."
            return
        }
        if s.name.trimmingCharacters(in: .whitespaces).isEmpty {
            s.name = "Player" + String(Int.random(in: 100...999))
        }
        do { try s.save() } catch { self.error = "Could not save settings: \(error.localizedDescription)"; return }
        s.applyCustomLayout()
        play()
    }
}

@_cdecl("muguet_physical_memory")
public func muguetPhysicalMemory() -> UInt64 { ProcessInfo.processInfo.physicalMemory }

private final class RefreshHint: NSObject {
    static let shared = RefreshHint()
    var link: CADisplayLink?
    private let condition = NSCondition()
    private var vsyncCount: UInt64 = 0
    var lockedPacingEnabled: Bool = false

    @objc func tick(_ link: CADisplayLink) {
        guard lockedPacingEnabled else { return }
        condition.lock()
        vsyncCount &+= 1
        condition.broadcast()
        condition.unlock()
    }

    func set(_ requested: UInt32, lockedPacing: Bool = false) {
        self.lockedPacingEnabled = lockedPacing
        let screenMax = Float(UIScreen.main.maximumFramesPerSecond)
        let maximum = max(120.0, screenMax)
        let rate = requested == 0 ? maximum : min(maximum, Float(requested))
        if link == nil {
            link = CADisplayLink(target: self, selector: #selector(tick(_:)))
            link?.add(to: .main, forMode: .common)
        }
        if lockedPacing {
            link?.preferredFrameRateRange = CAFrameRateRange(minimum: rate, maximum: rate, preferred: rate)
        } else {
            link?.preferredFrameRateRange = CAFrameRateRange(minimum: min(30, rate), maximum: rate, preferred: rate)
        }
    }

    func waitVsync(timeoutMs: UInt32) -> Bool {
        guard lockedPacingEnabled else { return true }
        condition.lock()
        let current = vsyncCount
        let deadline = Date(timeIntervalSinceNow: Double(timeoutMs) / 1000.0)
        var signaled = false
        while vsyncCount == current {
            if !condition.wait(until: deadline) {
                break
            }
            signaled = true
        }
        condition.unlock()
        return signaled
    }
}

@_cdecl("muguet_set_refresh_rate")
public func muguetSetRefreshRate(_ fps: UInt32) {
    let settings = Settings()
    DispatchQueue.main.async { RefreshHint.shared.set(fps, lockedPacing: settings.promotionSync) }
}

@_cdecl("muguet_wait_vsync")
public func muguetWaitVsync(_ timeoutMs: UInt32) -> Bool {
    return RefreshHint.shared.waitVsync(timeoutMs: timeoutMs)
}

@_cdecl("muguet_open_menu")
public func muguetOpenMenu() {
    DispatchQueue.main.async {
        guard let scene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
              let window = scene.windows.first(where: { $0.isKeyWindow }),
              let root = window.rootViewController else { return }
        var presenter = root
        while let child = presenter.presentedViewController { presenter = child }

        let alert = UIAlertController(title: "Muguet Menu", message: nil, preferredStyle: .alert)
        alert.addAction(UIAlertAction(title: "Tiếp tục chơi", style: .cancel))

        alert.addAction(UIAlertAction(title: "Chỉnh vị trí & kích thước nút (HUD)", style: .default, handler: { _ in
            let host = UIHostingController(rootView: HUDEditorView(settings: Settings()))
            host.modalPresentationStyle = .fullScreen
            presenter.present(host, animated: true)
        }))

        let isMp = muguet_is_multiplayer()
        if !isMp {
            alert.addAction(UIAlertAction(title: "Bảng lệnh cheat", style: .default, handler: { _ in
                let cheatHost = UIHostingController(rootView: CheatMenuView(onApply: { code in
                    code.withCString { muguet_apply_cheat($0) }
                }))
                cheatHost.modalPresentationStyle = .formSheet
                presenter.present(cheatHost, animated: true)
            }))
        }

        alert.addAction(UIAlertAction(title: "Cài đặt & Tùy chọn", style: .default, handler: { _ in
            weak var sheet: UIViewController?
            let host = UIHostingController(rootView: LauncherView(isOverlay: true, play: {
                sheet?.dismiss(animated: true)
                muguet_reload_settings()
            }))
            sheet = host
            host.modalPresentationStyle = .fullScreen
            host.isModalInPresentation = true
            presenter.present(host, animated: true)
        }))

        alert.addAction(UIAlertAction(title: "Thoát ứng dụng", style: .destructive, handler: { _ in
            exit(0)
        }))

        presenter.present(alert, animated: true)
    }
}

func shareDiagnostics() {
    let folder = configURL.deletingLastPathComponent()
    let logs = ["muguet-previous.log", "muguet-runtime.log"].map { folder.appendingPathComponent($0) }
        .filter { FileManager.default.fileExists(atPath: $0.path) }
    guard !logs.isEmpty,
          let scene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
          let window = scene.windows.first(where: { $0.isKeyWindow }),
          let root = window.rootViewController else { return }
    var presenter = root
    while let child = presenter.presentedViewController { presenter = child }
    let sheet = UIActivityViewController(activityItems: logs, applicationActivities: nil)
    if let popover = sheet.popoverPresentationController {
        popover.sourceView = presenter.view
        popover.sourceRect = CGRect(x: presenter.view.bounds.midX, y: presenter.view.bounds.midY, width: 1, height: 1)
    }
    presenter.present(sheet, animated: true)
}

func shareSaves() {
    let folder = configURL.deletingLastPathComponent().appendingPathComponent("saves")
    try? FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
    guard let scene = UIApplication.shared.connectedScenes.first as? UIWindowScene,
          let window = scene.windows.first(where: { $0.isKeyWindow }),
          let root = window.rootViewController else { return }
    var presenter = root
    while let child = presenter.presentedViewController { presenter = child }
    let sheet = UIActivityViewController(activityItems: [folder], applicationActivities: nil)
    if let popover = sheet.popoverPresentationController {
        popover.sourceView = presenter.view
        popover.sourceRect = CGRect(x: presenter.view.bounds.midX, y: presenter.view.bounds.midY, width: 1, height: 1)
    }
    presenter.present(sheet, animated: true)
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

@_cdecl("muguet_show_launcher")
public func muguetShowLauncher(_ viewController: UnsafeMutableRawPointer, _ done: @convention(c) () -> Void) {
    let parent = Unmanaged<UIViewController>.fromOpaque(viewController).takeUnretainedValue()
    weak var sheet: UIViewController?
    let host = UIHostingController(rootView: LauncherView(isOverlay: false, play: {
        if let sheet {
            sheet.dismiss(animated: true, completion: { done() })
        } else {
            done()
        }
    }))
    sheet = host
    host.modalPresentationStyle = .fullScreen
    host.isModalInPresentation = true
    parent.present(host, animated: false)
}
