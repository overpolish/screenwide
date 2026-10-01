// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <Metal/Metal.h>

/// Shared Inter regular body font for native OSC text.
NSFont *screenwide_osc_body_font(void);

@interface ScreenwideOscTextTexture : NSObject
@property(nonatomic, strong) id<MTLTexture> texture;
@property(nonatomic) NSSize size;
/// The uniform cell the texture stores each glyph in, centred: the widest
/// advance in the set. Quads sample a whole cell, so this is the quad width.
@property(nonatomic) CGFloat atlasGlyphWidth;
@property(nonatomic) CGFloat atlasGlyphUOffset;
@property(nonatomic) CGFloat atlasGlyphUWidth;
/// One `CGFloat` per cell: the glyph's own advance, which is what a readout
/// steps by on screen. Digits share one advance because the face is drawn
/// with tabular figures.
@property(nonatomic, strong) NSData *atlasAdvances;
@end

/// The cell `glyph` occupies in the atlas, or cell 0 for a character the
/// atlas has no cell for.
NSUInteger screenwide_osc_atlas_glyph_index(unichar glyph);
/// The advance of one cell, or the uniform cell width when the texture
/// carries no advances.
CGFloat screenwide_osc_atlas_advance(ScreenwideOscTextTexture *atlas,
                                     NSUInteger index);
/// The widest advance among `glyphs`. A field whose value changes under the
/// pointer, such as a hex colour, is laid out on this single pitch so its
/// columns cannot shuffle; fields that only ever hold digits and fixed
/// separators use the per-glyph advances instead.
CGFloat screenwide_osc_atlas_pitch(ScreenwideOscTextTexture *atlas,
                                   NSString *glyphs);
/// The on-screen width of `text` assembled out of atlas cells.
CGFloat screenwide_osc_atlas_text_width(ScreenwideOscTextTexture *atlas,
                                        NSString *text);

ScreenwideOscTextTexture *screenwide_osc_text_texture(
    id<MTLDevice> device, NSString *text, CGFloat scale,
    uint32_t light_mode, CGFloat font_size, CGFloat line_height);
ScreenwideOscTextTexture *screenwide_osc_mono_text_texture(
    id<MTLDevice> device, NSString *text, CGFloat scale,
    uint32_t light_mode, CGFloat font_size, CGFloat line_height);
ScreenwideOscTextTexture *screenwide_osc_mono_hex_atlas(
    id<MTLDevice> device, CGFloat scale, uint32_t light_mode,
    CGFloat font_size, CGFloat line_height);
