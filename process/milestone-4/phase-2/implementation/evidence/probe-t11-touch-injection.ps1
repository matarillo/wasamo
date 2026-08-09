# M4-Phase 2 T11 touch-injection probe.
#
# What this establishes. Claiming a WM_POINTER* message -- returning
# without reaching DefWindowProcW -- is what suppresses the mouse messages
# Windows would otherwise synthesize for a contact. That is the property
# stage 1's mutation witness W2 established no message-level test can see:
# SendMessageW-delivered pointer messages carry no real OS pointer id, so
# DefWindowProcW promotes nothing regardless of which arms claim which
# messages. The only way to see the suppression is a real OS pointer/touch
# contact, injected through the real input stack, delivered to a real
# window procedure -- which is what this script does.
#
# **What suppression is actually keyed on was measured wrong by an earlier
# version of this probe.** That version ran only two legs -- claim every
# member, claim none -- and the runtime's code comment inferred the middle
# ground from those two ends alone ("claim every member or promotion
# returns"). This script now measures the middle directly, per member and
# on a moving contact, and that inference was WRONG. This header must not
# be read as restating it.
#
# The measured rule: promotion is suppressed **per contact**, gated on the
# button-transition members alone. Claiming WM_POINTERDOWN or
# WM_POINTERUP -- either one, on its own -- suppresses the whole contact's
# promotion, including the WM_MOUSEMOVE an unclaimed WM_POINTERUPDATE
# would otherwise produce on a *moving* contact. Claiming only
# WM_POINTERENTER, or only WM_POINTERLEAVE, suppresses nothing. The
# per-member and moving-contact legs below measure this directly rather
# than inferring it, and the verdicts fail loudly if this machine
# disagrees.
#
# This is also what makes the shipped runtime's "claim all five members
# anyway" choice (window.rs's claims_pointer_message_without_acting and
# its WM_POINTERUP arm) a **deliberate** choice rather than a load-bearing
# one: the measured rule shows that dropping ENTER, UPDATE or LEAVE from
# the claimed set would not, on this machine, reawaken promotion, so
# claiming them anyway is about keeping the runtime's behaviour
# independent of a promotion rule the OS owns and may change -- it is not
# a requirement any verdict below depends on.
#
# It also records the screen-vs-client coordinate fact this task's
# conversion step exists for: WM_POINTER*'s lParam is in SCREEN
# coordinates, while the mouse messages the OS promotes from an unclaimed
# contact carry CLIENT coordinates. And it prints each pointer message's
# wParam HIGH WORD and whether its POINTER_MESSAGE_FLAG_PRIMARY (0x2000)
# bit is set, because the shipped runtime's WM_POINTERUP arm now gates
# dispatch on exactly that bit (window.rs's pointer_message_is_primary) --
# this is where a reader can see that an injected contact actually
# carries it.
#
# This script deliberately uses a plain instrumented window and NO wasamo
# code. Every result below is therefore a property of Windows on this
# machine, not of this runtime -- the same posture the T11 start gate's
# "eight measured facts" probe used (log.md SS T11).
#
# ONE invocation runs every leg in both tables below -- ten stationary-
# contact legs (one per claimed-member combination) and three
# moving-contact legs -- against the same window, back to back, so a
# reviewer runs one command and reads every message log and every
# verdict.
#
# Environment requirement: this needs an interactive desktop session --
# docs/notes/verification-environments.md's GUI / interactive class, not
# the headless-runtime-with-live-Compositor class the cargo integration
# suite runs in. Touch/pointer injection is desktop-scoped, not
# window-scoped: the injected contact goes to whatever window is at the
# screen point, which is why this script asserts WindowFromPoint is its
# own window immediately before injecting, and FAILS LOUDLY (non-zero
# exit) rather than continuing if another window is on top -- a silent
# continue there would inject into the wrong window and produce a log that
# looks like a clean result for the wrong reason.
#
# Run with Windows PowerShell 5.1, not PowerShell 7 (pwsh) -- this uses
# System.Windows.Forms in a single-threaded apartment, which pwsh's default
# apartment does not guarantee:
#
#   $env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe -NoProfile -STA -ExecutionPolicy Bypass -File probe-t11-touch-injection.ps1
#
# Two injection-side traps this script has already solved (T11 start-gate
# fact 6), both of which otherwise fail with a bare ERROR_INVALID_PARAMETER
# and no diagnostic:
#   - `pressure` must be in the touch range 0..1024 (a pen-ish value like
#     32000 is rejected).
#   - POINTER_TOUCH_INFO must be built field-by-field INSIDE the C# layer.
#     PowerShell assignment to a *nested* value-type field mutates a copy,
#     so a struct built from PowerShell reaches the OS still zeroed.

$ErrorActionPreference = "Stop"

$src = @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Windows.Forms;
using System.Drawing;

public static class Native {
    [DllImport("user32.dll", SetLastError=true)]
    public static extern bool SetProcessDpiAwarenessContext(IntPtr value);
    [DllImport("user32.dll", SetLastError=true)]
    public static extern uint GetDpiForWindow(IntPtr hwnd);
    [DllImport("user32.dll", SetLastError=true)]
    public static extern bool InitializeTouchInjection(uint maxCount, uint dwMode);
    [DllImport("user32.dll", SetLastError=true)]
    static extern bool InjectTouchInput(uint count, [In] POINTER_TOUCH_INFO[] contacts);
    [DllImport("user32.dll", SetLastError=true)]
    public static extern bool SetForegroundWindow(IntPtr hwnd);
    [DllImport("user32.dll", SetLastError=true)]
    public static extern bool GetClientRect(IntPtr hwnd, out RECT r);
    [DllImport("user32.dll", SetLastError=true)]
    public static extern bool ClientToScreen(IntPtr hwnd, ref POINT p);
    [DllImport("user32.dll", SetLastError=true)]
    public static extern IntPtr WindowFromPoint(POINT p);
    [DllImport("user32.dll", SetLastError=true)]
    public static extern bool SetCursorPos(int x, int y);

    [StructLayout(LayoutKind.Sequential)]
    public struct POINT { public int x; public int y; }
    [StructLayout(LayoutKind.Sequential)]
    public struct RECT { public int left; public int top; public int right; public int bottom; }

    [StructLayout(LayoutKind.Sequential)]
    public struct POINTER_INFO {
        public uint pointerType;
        public uint pointerId;
        public uint frameId;
        public uint pointerFlags;
        public IntPtr sourceDevice;
        public IntPtr hwndTarget;
        public POINT ptPixelLocation;
        public POINT ptHimetricLocation;
        public POINT ptPixelLocationRaw;
        public POINT ptHimetricLocationRaw;
        public uint dwTime;
        public uint historyCount;
        public int inputData;
        public uint dwKeyStates;
        public ulong PerformanceCount;
        public int ButtonChangeType;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct POINTER_TOUCH_INFO {
        public POINTER_INFO pointerInfo;
        public uint touchFlags;
        public uint touchMask;
        public RECT rcContact;
        public RECT rcContactRaw;
        public uint orientation;
        public uint pressure;
    }

    public const uint PT_TOUCH = 2;
    public const uint INRANGE = 0x00000002;
    public const uint INCONTACT = 0x00000004;
    public const uint DOWN = 0x00010000;
    public const uint UPDATE = 0x00020000;
    public const uint UP = 0x00040000;

    // Built entirely in C#: PowerShell assignment to a *nested* value-type
    // field (pointerInfo.ptPixelLocation.x, say) mutates a copy and the OS
    // receives a still-zeroed struct. See this script's header, trap 2.
    static POINTER_TOUCH_INFO MakeContact(int x, int y, uint flags) {
        POINTER_TOUCH_INFO c = new POINTER_TOUCH_INFO();
        c.pointerInfo.pointerType = PT_TOUCH;
        c.pointerInfo.pointerId = 0;
        c.pointerInfo.pointerFlags = flags;
        c.pointerInfo.ptPixelLocation.x = x;
        c.pointerInfo.ptPixelLocation.y = y;
        c.touchFlags = 0;
        c.touchMask = 0x00000007; // CONTACTAREA | ORIENTATION | PRESSURE
        c.rcContact.left = x - 2; c.rcContact.top = y - 2;
        c.rcContact.right = x + 2; c.rcContact.bottom = y + 2;
        c.orientation = 90;
        // Touch range is 0..1024 (trap 1). A pen-ish 32000 is rejected with
        // a bare ERROR_INVALID_PARAMETER and no further diagnostic.
        c.pressure = 512;
        return c;
    }

    public static string InjectTouch(int x, int y, uint flags) {
        POINTER_TOUCH_INFO[] a = new POINTER_TOUCH_INFO[1];
        a[0] = MakeContact(x, y, flags);
        bool ok = InjectTouchInput(1, a);
        // GetLastWin32Error is only meaningful when the call actually
        // failed -- printed beside a success it is frequently a stale
        // code left over from an unrelated earlier call, and reads like a
        // defect to a reviewer even though nothing failed.
        if (ok) { return "ok=True"; }
        int err = Marshal.GetLastWin32Error();
        return "ok=False err=0x" + err.ToString("X8");
    }
}

public class ProbeForm : Form {
    public List<string> Log = new List<string>();

    // The claimed set for the CURRENT leg: message codes this window will
    // return LRESULT(0) for without reaching DefWindowProcW. Generalised
    // from the superseded all-or-nothing `HandlePointer` bool so each leg
    // can claim an arbitrary subset -- the shape Run-StationaryLeg /
    // Run-MovingLeg below populate before every leg and WndProc tests with
    // Contains.
    public List<int> Claimed = new List<int>();

    public const int WM_POINTERUPDATE = 0x0245;
    public const int WM_POINTERDOWN = 0x0246;
    public const int WM_POINTERUP = 0x0247;
    public const int WM_POINTERENTER = 0x0249;
    public const int WM_POINTERLEAVE = 0x024A;

    static string MsgName(int m) {
        switch (m) {
            case 0x0245: return "WM_POINTERUPDATE";
            case 0x0246: return "WM_POINTERDOWN";
            case 0x0247: return "WM_POINTERUP";
            case 0x0249: return "WM_POINTERENTER";
            case 0x024A: return "WM_POINTERLEAVE";
            case 0x0240: return "WM_TOUCH";
            case 0x0200: return "WM_MOUSEMOVE";
            case 0x0201: return "WM_LBUTTONDOWN";
            case 0x0202: return "WM_LBUTTONUP";
            default: return null;
        }
    }

    protected override void WndProc(ref Message m) {
        string n = MsgName(m.Msg);
        if (n != null) {
            int lo = (short)((long)m.LParam & 0xFFFF);
            int hi = (short)(((long)m.LParam >> 16) & 0xFFFF);
            string line = n + " lparam=(" + lo + "," + hi + ") pointerId=" + ((long)m.WParam & 0xFFFF);
            bool isPointerMsg = n.StartsWith("WM_POINTER");
            if (isPointerMsg) {
                // Windows packs a WM_POINTER* message's condition flags
                // into the HIGH word of wParam (POINTER_MESSAGE_FLAG_*,
                // one bit per condition) and the pointer id into the low
                // word. PRIMARY is 0x2000. Printed here because the
                // shipped runtime's WM_POINTERUP arm now gates dispatch on
                // this bit (window.rs's pointer_message_is_primary) -- so
                // a reader can see, on a real injected contact, that the
                // bit is actually set.
                int hiword = (int)(((long)m.WParam >> 16) & 0xFFFF);
                bool primary = (hiword & 0x2000) != 0;
                line += " wparam_hiword=0x" + hiword.ToString("X4") + " primary=" + (primary ? "YES" : "NO");
            }
            Log.Add(line);
            if (isPointerMsg && Claimed.Contains(m.Msg)) {
                m.Result = IntPtr.Zero;
                return; // do NOT call base: this is what suppresses promotion for a claimed member
            }
        }
        base.WndProc(ref m);
    }
}
'@

Add-Type -TypeDefinition $src -ReferencedAssemblies System.Windows.Forms, System.Drawing -ErrorAction Stop

$PMV2 = [IntPtr](-4)
$dpiOk = [Native]::SetProcessDpiAwarenessContext($PMV2)
Write-Host "dpi_awareness_set=$dpiOk"

$form = New-Object ProbeForm
$form.Text = "wasamo T11 touch-injection probe"
$form.StartPosition = "Manual"
$form.Location = New-Object System.Drawing.Point(200, 200)
$form.Size = New-Object System.Drawing.Size(500, 400)
$form.TopMost = $true
$form.Show()
[System.Windows.Forms.Application]::DoEvents()
$null = [Native]::SetForegroundWindow($form.Handle)
Start-Sleep -Milliseconds 400
[System.Windows.Forms.Application]::DoEvents()

$hwnd = $form.Handle
Write-Host "hwnd=$hwnd dpi_for_window=$([Native]::GetDpiForWindow($hwnd))"

$rc = New-Object Native+RECT
$null = [Native]::GetClientRect($hwnd, [ref]$rc)
Write-Host "client_rect=(0,0)-($($rc.right),$($rc.bottom))"

# The injection point, expressed both ways: the CLIENT point this probe
# intends to tap, and the SCREEN point ClientToScreen maps it to. Both are
# printed so the screen-vs-client verdict below has concrete numbers to
# point at, not just an assertion.
$clientPt = New-Object Native+POINT
$clientPt.x = [int]($rc.right / 2); $clientPt.y = [int]($rc.bottom / 2)
$screenPt = New-Object Native+POINT
$screenPt.x = $clientPt.x; $screenPt.y = $clientPt.y
$null = [Native]::ClientToScreen($hwnd, [ref]$screenPt)
Write-Host "intended_client_point=($($clientPt.x),$($clientPt.y)) -> screen_point=($($screenPt.x),$($screenPt.y))"

# Injection is desktop-scoped, not window-scoped (T11 start-gate fact 5):
# the contact goes to whatever window is at the screen point. A silent
# continue past a mismatch here would inject into the wrong window and
# produce a log that looks clean for the wrong reason, so this FAILS
# LOUDLY (non-zero exit) instead.
$under = [Native]::WindowFromPoint($screenPt)
Write-Host "window_from_point=$under probe_hwnd=$hwnd"
if ($under -ne $hwnd) {
    Write-Error "WindowFromPoint($($screenPt.x),$($screenPt.y)) returned $under, not this probe's own window ($hwnd) -- another window is on top of the injection point. Move it and re-run; injecting anyway would tap the wrong window."
    $form.Close()
    exit 1
}

# The moving-contact legs walk up to 30px further along X from $screenPt
# (three 10px steps). Checked once, up front, alongside the guard above:
# the furthest point a moving leg ever injects at must also be over this
# window, or a moving leg would silently tap whatever sits beyond it.
$farPt = New-Object Native+POINT
$farPt.x = $screenPt.x + 30
$farPt.y = $screenPt.y
$underFar = [Native]::WindowFromPoint($farPt)
Write-Host "window_from_point(moving leg far end)=$underFar probe_hwnd=$hwnd"
if ($underFar -ne $hwnd) {
    Write-Error "WindowFromPoint($($farPt.x),$($farPt.y)) returned $underFar, not this probe's own window ($hwnd) -- the moving-contact legs' furthest point (+30px along X) is not over this window. Move it and re-run."
    $form.Close()
    exit 1
}

function Pump([int]$ms) {
    $end = (Get-Date).AddMilliseconds($ms)
    while ((Get-Date) -lt $end) { [System.Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 10 }
}

# A fixed screen position well clear of the probe window (which sits at
# (200,200)-(700,600)), used to park the physical cursor before every leg.
$PARK_SCREEN_X = 5
$PARK_SCREEN_Y = 5

$DOWNFLAGS   = [uint32](0x00010000 -bor 0x2 -bor 0x4)  # DOWN | INRANGE | INCONTACT
$UPFLAGS     = [uint32]0x00040000                       # UP
$UPDATEFLAGS = [uint32](0x00020000 -bor 0x2 -bor 0x4)  # UPDATE | INRANGE | INCONTACT (0x00020006)

if (-not [Native]::InitializeTouchInjection(1, 3)) {
    Write-Error "InitializeTouchInjection failed (err=0x$([Runtime.InteropServices.Marshal]::GetLastWin32Error().ToString('X8'))) -- touch injection is unavailable in this session"
    $form.Close()
    exit 1
}

# ── Message-code constants and claimed-set legs ────────────────────────────
$MSG_ENTER  = [ProbeForm]::WM_POINTERENTER
$MSG_DOWN   = [ProbeForm]::WM_POINTERDOWN
$MSG_UPDATE = [ProbeForm]::WM_POINTERUPDATE
$MSG_UP     = [ProbeForm]::WM_POINTERUP
$MSG_LEAVE  = [ProbeForm]::WM_POINTERLEAVE

$MSG_NAME_BY_CODE = @{
    $MSG_ENTER  = "ENTER"
    $MSG_DOWN   = "DOWN"
    $MSG_UPDATE = "UPDATE"
    $MSG_UP     = "UP"
    $MSG_LEAVE  = "LEAVE"
}

function Describe-Claimed([int[]]$ClaimedSet) {
    if (-not $ClaimedSet -or $ClaimedSet.Count -eq 0) { return "(none)" }
    ($ClaimedSet | ForEach-Object { $MSG_NAME_BY_CODE[$_] }) -join "+"
}

# Table 1 (stationary contact: inject DOWN then UP at the same point).
# Expected promoted-mouse-message counts are the measured facts this
# task's independent-review correction recorded.
$stationaryLegs = @(
    @{ Id = "all5";      Name = "all five claimed";       Claimed = @($MSG_ENTER, $MSG_DOWN, $MSG_UPDATE, $MSG_UP, $MSG_LEAVE); Expected = 0 }
    @{ Id = "none";      Name = "none claimed";           Claimed = @();                                                       Expected = 4 }
    @{ Id = "allButDown"; Name = "all but DOWN claimed";  Claimed = @($MSG_ENTER, $MSG_UPDATE, $MSG_UP, $MSG_LEAVE);            Expected = 0 }
    @{ Id = "allButUp";   Name = "all but UP claimed";    Claimed = @($MSG_ENTER, $MSG_DOWN, $MSG_UPDATE, $MSG_LEAVE);          Expected = 0 }
    @{ Id = "allButEnter"; Name = "all but ENTER claimed"; Claimed = @($MSG_DOWN, $MSG_UPDATE, $MSG_UP, $MSG_LEAVE);            Expected = 0 }
    @{ Id = "downUp";    Name = "DOWN+UP only claimed";   Claimed = @($MSG_DOWN, $MSG_UP);                                     Expected = 0 }
    @{ Id = "enterOnly"; Name = "ENTER only claimed";     Claimed = @($MSG_ENTER);                                             Expected = 4 }
    @{ Id = "leaveOnly"; Name = "LEAVE only claimed";     Claimed = @($MSG_LEAVE);                                             Expected = 4 }
    @{ Id = "downOnly";  Name = "DOWN only claimed";      Claimed = @($MSG_DOWN);                                              Expected = 0 }
    @{ Id = "upOnly";    Name = "UP only claimed";        Claimed = @($MSG_UP);                                                Expected = 0 }
)

# Table 2 (moving contact: inject DOWN, three UPDATE frames 10px apart,
# then UP -- POINTER_FLAG_UPDATE|INRANGE|INCONTACT is 0x00020006).
$movingLegs = @(
    @{ Id = "all5Moving";   Name = "all five claimed (moving)";                          Claimed = @($MSG_ENTER, $MSG_DOWN, $MSG_UPDATE, $MSG_UP, $MSG_LEAVE); Expected = 0 }
    @{ Id = "downUpMoving"; Name = "DOWN+UP only claimed, UPDATE unclaimed (moving)";     Claimed = @($MSG_DOWN, $MSG_UP);                                     Expected = 0 }
    @{ Id = "noneMoving";   Name = "none claimed (moving)";                              Claimed = @();                                                       Expected = 6 }
)

# A message counts as "promoted for this contact" if it is a mouse message
# appearing anywhere in this leg's log. No position/content filtering is
# needed (unlike the superseded two-leg version): the physical cursor is
# parked off-window and the log is cleared immediately before every leg
# (Run-StationaryLeg / Run-MovingLeg below), so every mouse message that
# appears during a leg's window is attributable to that leg's own
# injected contact, not to incidental cursor traffic -- and a moving
# contact's promoted messages carry a moving client point that a single
# fixed-point content check could not match anyway.
function Count-PromotedMouseMessages([string[]]$LogLines) {
    $n = 0
    foreach ($l in $LogLines) {
        if ($l.StartsWith("WM_MOUSEMOVE") -or $l.StartsWith("WM_LBUTTONDOWN") -or $l.StartsWith("WM_LBUTTONUP")) { $n++ }
    }
    return $n
}

# ── Run one stationary leg: inject a stationary tap, collect the log ──────
function Run-StationaryLeg($Leg) {
    $form.Claimed.Clear()
    foreach ($m in $Leg.Claimed) { $form.Claimed.Add($m) }

    # Park the physical cursor off-window and let it settle BEFORE
    # clearing the log and injecting. A stray WM_MOUSEMOVE from wherever
    # the operator's mouse happens to be is indistinguishable from a
    # promoted one to a naive counter, so the artifact must not depend on
    # where the operator left the mouse -- this makes every leg start
    # from the same cursor state regardless of prior operator input.
    [Native]::SetCursorPos($PARK_SCREEN_X, $PARK_SCREEN_Y) | Out-Null
    Pump 200
    $form.Log.Clear()

    Write-Host ""
    Write-Host "--- stationary leg: $($Leg.Name) (claimed=$(Describe-Claimed $Leg.Claimed)) ---"
    $downResult = [Native]::InjectTouch($screenPt.x, $screenPt.y, $DOWNFLAGS)
    Write-Host "  inject down: $downResult"
    Pump 150
    $upResult = [Native]::InjectTouch($screenPt.x, $screenPt.y, $UPFLAGS)
    Write-Host "  inject up:   $upResult"
    Pump 300

    Write-Host "  message sequence:"
    foreach ($line in $form.Log) { Write-Host "    $line" }

    $log = $form.Log.ToArray()
    $promoted = Count-PromotedMouseMessages $log
    $match = if ($promoted -eq $Leg.Expected) { "matches table" } else { "DOES NOT MATCH table expectation of $($Leg.Expected)" }
    Write-Host "  promoted mouse messages (stationary contact): $promoted ($match)"

    return @{ Log = $log; Promoted = $promoted; Expected = $Leg.Expected; Name = $Leg.Name }
}

# ── Run one moving leg: inject DOWN, three 10px UPDATE steps, then UP ─────
function Run-MovingLeg($Leg) {
    $form.Claimed.Clear()
    foreach ($m in $Leg.Claimed) { $form.Claimed.Add($m) }

    [Native]::SetCursorPos($PARK_SCREEN_X, $PARK_SCREEN_Y) | Out-Null
    Pump 200
    $form.Log.Clear()

    Write-Host ""
    Write-Host "--- moving leg: $($Leg.Name) (claimed=$(Describe-Claimed $Leg.Claimed)) ---"
    $downResult = [Native]::InjectTouch($screenPt.x, $screenPt.y, $DOWNFLAGS)
    Write-Host "  inject down at ($($screenPt.x),$($screenPt.y)): $downResult"
    Pump 100

    $mx = $screenPt.x
    $my = $screenPt.y
    for ($i = 1; $i -le 3; $i++) {
        $mx = $screenPt.x + ($i * 10)
        $updResult = [Native]::InjectTouch($mx, $my, $UPDATEFLAGS)
        Write-Host "  inject update $i at ($mx,$my): $updResult"
        Pump 100
    }

    $upResult = [Native]::InjectTouch($mx, $my, $UPFLAGS)
    Write-Host "  inject up at ($mx,$my):   $upResult"
    Pump 300

    Write-Host "  message sequence:"
    foreach ($line in $form.Log) { Write-Host "    $line" }

    $log = $form.Log.ToArray()
    $promoted = Count-PromotedMouseMessages $log
    $match = if ($promoted -eq $Leg.Expected) { "matches table" } else { "DOES NOT MATCH table expectation of $($Leg.Expected)" }
    Write-Host "  promoted mouse messages (moving contact): $promoted ($match)"

    return @{ Log = $log; Promoted = $promoted; Expected = $Leg.Expected; Name = $Leg.Name }
}

Write-Host ""
Write-Host "=== stationary-contact legs (table 1) ==="
$stationaryResults = @{}
foreach ($leg in $stationaryLegs) {
    $stationaryResults[$leg.Id] = Run-StationaryLeg $leg
}

Write-Host ""
Write-Host "=== moving-contact legs (table 2) ==="
$movingResults = @{}
foreach ($leg in $movingLegs) {
    $movingResults[$leg.Id] = Run-MovingLeg $leg
}

# ── Verdicts ────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "=== verdicts ==="

function Test-HasPrefix($LogLines, [string]$Prefix) {
    foreach ($l in $LogLines) { if ($l.StartsWith($Prefix)) { return $true } }
    return $false
}

# Claim 1: message family delivered -- both legs must show real
# WM_POINTER* traffic, or the injection did not reach this window at all.
# Reuses the "all five claimed" / "none claimed" stationary legs, the same
# two legs the superseded version of this script ran as its only legs.
$claimLog = $stationaryResults["all5"].Log
$passthroughLog = $stationaryResults["none"].Log
$claimPointerSeen = Test-HasPrefix $claimLog "WM_POINTERDOWN" -or (Test-HasPrefix $claimLog "WM_POINTERUP")
$passthroughPointerSeen = Test-HasPrefix $passthroughLog "WM_POINTERDOWN" -or (Test-HasPrefix $passthroughLog "WM_POINTERUP")
$msgFamilyOk = $claimPointerSeen -and $passthroughPointerSeen
Write-Host "1. message family delivered: all5_leg_saw_pointer_messages=$claimPointerSeen none_leg_saw_pointer_messages=$passthroughPointerSeen -> $(if ($msgFamilyOk) { 'PASS' } else { 'FAIL' })"

# Claim 2: screen-vs-client. WM_POINTER*'s lParam must read back as the
# SCREEN point this script injected at, not the CLIENT point it intended to
# tap (the two differ because the window sits at (200,200), off the
# desktop origin). The promoted WM_MOUSEMOVE / WM_LBUTTONDOWN in the
# none-claimed leg, by contrast, must read back as the CLIENT point.
$screenLine = $passthroughLog | Where-Object { $_.StartsWith("WM_POINTERDOWN") } | Select-Object -First 1
$mouseLine = $passthroughLog | Where-Object { $_.StartsWith("WM_LBUTTONDOWN") } | Select-Object -First 1
Write-Host "2. screen-vs-client: WM_POINTERDOWN lparam line = '$screenLine' (expected to match screen_point ($($screenPt.x),$($screenPt.y)))"
Write-Host "   screen-vs-client: WM_LBUTTONDOWN lparam line = '$mouseLine' (expected to match client_point ($($clientPt.x),$($clientPt.y)))"
$screenVsClientOk = ($screenLine -and $screenLine.Contains("($($screenPt.x),$($screenPt.y))")) -and
                     ($mouseLine -and $mouseLine.Contains("($($clientPt.x),$($clientPt.y))"))
Write-Host "   -> $(if ($screenVsClientOk) { 'PASS' } else { 'FAIL (see raw logs above; a probe window at the desktop origin would make this verdict vacuous -- this one sits at (200,200))' })"

# Claim 3: the measured per-member / moving-contact suppression rule.
# "Suppresses" / "promotes" are read off Count-PromotedMouseMessages as
# ==0 / >0 -- the binary property the rule is actually about -- not an
# exact match to the table's numbers (those are cross-checked separately
# below, informationally, for every leg run above).
function Suppressed($LegResult) { $LegResult.Promoted -eq 0 }
function Promoted($LegResult) { $LegResult.Promoted -gt 0 }

$v3a = Suppressed $stationaryResults["all5"]
Write-Host "3a. all-claimed suppresses: promoted=$($stationaryResults["all5"].Promoted) -> $(if ($v3a) { 'PASS' } else { 'FAIL' })"

$v3b = Promoted $stationaryResults["none"]
Write-Host "3b. none-claimed promotes: promoted=$($stationaryResults["none"].Promoted) -> $(if ($v3b) { 'PASS' } else { 'FAIL' })"

$v3c = (Suppressed $stationaryResults["downOnly"]) -and (Suppressed $stationaryResults["upOnly"])
Write-Host "3c. DOWN-only and UP-only each suppress: DOWN-only promoted=$($stationaryResults["downOnly"].Promoted), UP-only promoted=$($stationaryResults["upOnly"].Promoted) -> $(if ($v3c) { 'PASS' } else { 'FAIL' })"

$v3d = (Promoted $stationaryResults["enterOnly"]) -and (Promoted $stationaryResults["leaveOnly"])
Write-Host "3d. ENTER-only and LEAVE-only each promote: ENTER-only promoted=$($stationaryResults["enterOnly"].Promoted), LEAVE-only promoted=$($stationaryResults["leaveOnly"].Promoted) -> $(if ($v3d) { 'PASS' } else { 'FAIL' })"

$v3e = Suppressed $movingResults["downUpMoving"]
Write-Host "3e. moving contact, DOWN+UP claimed / UPDATE unclaimed, still suppresses: promoted=$($movingResults["downUpMoving"].Promoted) -> $(if ($v3e) { 'PASS' } else { 'FAIL' })"

$measuredRuleOk = $v3a -and $v3b -and $v3c -and $v3d -and $v3e

# Informational cross-check against every leg's table expectation
# (including the six legs no verdict above names individually) -- these do
# not gate OVERALL, but a mismatch here is worth reporting: the six extra
# stationary legs are all entailed by the rule 3a-3e already assert (each
# claims DOWN and/or UP), so a mismatch among them would itself be a
# finding worth surfacing even though it does not fail the run.
Write-Host ""
Write-Host "=== table cross-check (informational; does not gate OVERALL) ==="
$anyTableMismatch = $false
foreach ($key in $stationaryResults.Keys) {
    $r = $stationaryResults[$key]
    if ($r.Promoted -ne $r.Expected) {
        $anyTableMismatch = $true
        Write-Host "  MISMATCH: stationary '$($r.Name)': table expects $($r.Expected), measured $($r.Promoted)"
    }
}
foreach ($key in $movingResults.Keys) {
    $r = $movingResults[$key]
    if ($r.Promoted -ne $r.Expected) {
        $anyTableMismatch = $true
        Write-Host "  MISMATCH: moving '$($r.Name)': table expects $($r.Expected), measured $($r.Promoted)"
    }
}
if (-not $anyTableMismatch) {
    Write-Host "  no mismatches: every leg's measured promoted-mouse-message count matches this script's header table."
}

Write-Host ""
$overallOk = $msgFamilyOk -and $screenVsClientOk -and $measuredRuleOk
if ($overallOk) {
    Write-Host "OVERALL: PASS -- promotion is suppressed per contact, gated on the button-transition members (DOWN/UP) alone; claiming only ENTER or only LEAVE suppresses nothing; WM_POINTER* arrives in screen space where the promoted mouse messages arrive in client space."
} else {
    Write-Host "OVERALL: FAIL -- see the verdict lines above."
}

$form.Close()
if (-not $overallOk) { exit 1 }
exit 0
