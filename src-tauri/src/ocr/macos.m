#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>
#import <Vision/Vision.h>

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

static char *lr_dup_nsstring(NSString *s) {
    if (!s) {
        return NULL;
    }
    const char *utf8 = [s UTF8String];
    if (!utf8) {
        return NULL;
    }
    return strdup(utf8);
}

char *linguaray_vision_ocr(const uint8_t *bytes, size_t len, char **err_out) {
    @autoreleasepool {
        if (err_out) {
            *err_out = NULL;
        }
        if (!bytes || len == 0) {
            if (err_out) {
                *err_out = strdup("empty image");
            }
            return NULL;
        }
        NSData *data = [NSData dataWithBytes:bytes length:len];
        NSImage *image = [[NSImage alloc] initWithData:data];
        if (!image) {
            if (err_out) {
                *err_out = strdup("image decode failed");
            }
            return NULL;
        }
        CGImageRef cg = [image CGImageForProposedRect:NULL context:nil hints:nil];
        if (!cg) {
            if (err_out) {
                *err_out = strdup("cgimage failed");
            }
            return NULL;
        }
        VNImageRequestHandler *handler =
            [[VNImageRequestHandler alloc] initWithCGImage:cg options:@{}];
        __block NSMutableString *acc = [NSMutableString string];
        VNRecognizeTextRequest *req =
            [[VNRecognizeTextRequest alloc] initWithCompletionHandler:^(VNRequest *request, NSError *error) {
                if (error) {
                    return;
                }
                for (VNRecognizedTextObservation *obs in request.results) {
                    VNRecognizedText *top = [[obs topCandidates:1] firstObject];
                    if (top.string.length == 0) {
                        continue;
                    }
                    if (acc.length > 0) {
                        [acc appendString:@" "];
                    }
                    [acc appendString:top.string];
                }
            }];
        req.recognitionLevel = VNRequestTextRecognitionLevelAccurate;
        req.usesLanguageCorrection = YES;
        NSError *perfErr = nil;
        if (![handler performRequests:@[req] error:&perfErr]) {
            if (err_out) {
                *err_out = lr_dup_nsstring(perfErr.localizedDescription ?: @"vision failed");
            }
            return NULL;
        }
        if (acc.length == 0) {
            return strdup("");
        }
        return lr_dup_nsstring(acc);
    }
}

void linguaray_free(void *p) {
    free(p);
}
