#import <AppKit/AppKit.h>
#import <QuartzCore/QuartzCore.h>

@interface NSPasteboard (Brasa)
- (NSArray *)getFilePaths;
@end

/// BrasaHostView is the Content view of a Brasa window.
// It is backed by a Metal CALayer.
@interface BrasaHostView : NSView <CALayerDelegate, NSTextInputClient>
- (BrasaHostView *)initWithFrame:(NSRect)frame
                    metalDevice:(id)metalDevice
             enableTitlebarDrag:(BOOL)enableTitlebarDrag
                       testMode:(BOOL)testMode;
- (void)setAsyncCallback:(BOOL)shouldAsync;
- (void)setPresentsWithTransaction:(BOOL)presentsWithTransaction;
- (BOOL)keyDownImpl:(NSEvent *)event;
@end
