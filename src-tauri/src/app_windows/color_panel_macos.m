// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <math.h>

// The system Colours panel, driven straight from the web UI's colour wells.
// WebKit's own `<input type="color">` popover is a WebKit window we cannot
// place or theme, so on macOS the well asks for this instead and receives the
// colour back as it is dragged.

typedef void (*ColorPanelChange)(const char *hex, void *context);
typedef void (*ColorPanelClose)(void *context);

// The panel is a single shared system object, so one well owns it at a time.
// Showing again while it is open just retargets these.
static ColorPanelChange gOnChange = NULL;
static ColorPanelClose gOnClose = NULL;
static void *gContext = NULL;

static NSColor *colorFromHex(const char *hex) {
  if (hex == NULL) return nil;
  NSString *text = [@(hex) stringByTrimmingCharactersInSet:
                               [NSCharacterSet characterSetWithCharactersInString:@"#"]];
  if (text.length != 6) return nil;
  unsigned int packed = 0;
  if (![[NSScanner scannerWithString:text] scanHexInt:&packed]) return nil;
  return [NSColor colorWithSRGBRed:(CGFloat)((packed >> 16) & 0xFF) / 255.0
                             green:(CGFloat)((packed >> 8) & 0xFF) / 255.0
                              blue:(CGFloat)(packed & 0xFF) / 255.0
                             alpha:1.0];
}

@interface ScreenwideColorPanelTarget : NSObject
- (void)changeColor:(id)sender;
- (void)panelWillClose:(NSNotification *)notification;
@end

@implementation ScreenwideColorPanelTarget

- (void)changeColor:(id)sender {
  if (gOnChange == NULL) return;
  NSColor *color = [NSColorPanel sharedColorPanel].color;
  NSColor *srgb = [color colorUsingColorSpace:NSColorSpace.sRGBColorSpace];
  if (srgb == nil) return;
  // Rounding, not truncating: 0.999 is white, not #FEFEFE.
  NSString *hex = [NSString stringWithFormat:@"#%02lX%02lX%02lX",
                                             (unsigned long)lround(srgb.redComponent * 255.0),
                                             (unsigned long)lround(srgb.greenComponent * 255.0),
                                             (unsigned long)lround(srgb.blueComponent * 255.0)];
  gOnChange(hex.UTF8String, gContext);
}

- (void)panelWillClose:(NSNotification *)notification {
  ColorPanelClose onClose = gOnClose;
  void *context = gContext;
  gOnChange = NULL;
  gOnClose = NULL;
  gContext = NULL;
  if (onClose != NULL) onClose(context);
}

@end

static ScreenwideColorPanelTarget *panelTarget(void) {
  static ScreenwideColorPanelTarget *target = nil;
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    target = [ScreenwideColorPanelTarget new];
  });
  return target;
}

// Must be called on the main thread. Sync Tauri commands already run there.
void screenwide_color_panel_show(const char *hex, ColorPanelChange onChange, ColorPanelClose onClose,
                                 void *context) {
  ScreenwideColorPanelTarget *target = panelTarget();
  NSColorPanel *panel = [NSColorPanel sharedColorPanel];

  static dispatch_once_t once;
  dispatch_once(&once, ^{
    [NSNotificationCenter.defaultCenter addObserver:target
                                           selector:@selector(panelWillClose:)
                                               name:NSWindowWillCloseNotification
                                             object:panel];
  });

  gOnChange = onChange;
  gOnClose = onClose;
  gContext = context;

  panel.showsAlpha = NO;
  panel.continuous = YES;
  panel.target = target;
  panel.action = @selector(changeColor:);
  NSColor *color = colorFromHex(hex);
  if (color != nil) panel.color = color;

  // Screenwide's own chrome (tool panels, the recording dock, Glide overlays)
  // sits on custom window levels up to 34, well above the Colours panel's
  // default floating level, so it would otherwise open behind the very panel
  // that asked for it. The app is left alone: the panel is ordered front
  // without activating, so a non-activating tool panel keeps its foreground.
  panel.level = NSStatusWindowLevel + 10;
  [panel orderFront:nil];
}

void screenwide_color_panel_close(void) {
  [[NSColorPanel sharedColorPanel] orderOut:nil];
}
