#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

static NSSpeechSynthesizer *lr_synth(void) {
    static NSSpeechSynthesizer *syn;
    static dispatch_once_t once;
    dispatch_once(&once, ^{
        syn = [[NSSpeechSynthesizer alloc] init];
    });
    return syn;
}

char *linguaray_tts_list_voices(char **err_out) {
    @autoreleasepool {
        if (err_out) {
            *err_out = NULL;
        }
        NSArray *voices = [NSSpeechSynthesizer availableVoices];
        NSMutableArray<NSString *> *ids = [NSMutableArray array];
        for (NSString *v in voices) {
            if (v.length) {
                [ids addObject:v];
            }
        }
        NSError *err = nil;
        NSData *json = [NSJSONSerialization dataWithJSONObject:ids options:0 error:&err];
        if (!json) {
            if (err_out) {
                *err_out = strdup("voice list json failed");
            }
            return NULL;
        }
        return strndup((const char *)json.bytes, json.length);
    }
}

int linguaray_tts_speak(const char *text, const char *voice_id, char **err_out) {
    @autoreleasepool {
        if (err_out) {
            *err_out = NULL;
        }
        if (!text || text[0] == 0) {
            if (err_out) {
                *err_out = strdup("empty text");
            }
            return -1;
        }
        NSSpeechSynthesizer *syn = lr_synth();
        if (voice_id && voice_id[0]) {
            NSString *vid = [NSString stringWithUTF8String:voice_id];
            if (![syn setVoice:vid]) {
                if (err_out) {
                    *err_out = strdup("unknown voice");
                }
                return -1;
            }
        }
        NSString *s = [NSString stringWithUTF8String:text];
        if (![syn startSpeakingString:s]) {
            if (err_out) {
                *err_out = strdup("speak failed");
            }
            return -1;
        }
        return 0;
    }
}

void linguaray_tts_stop(void) {
    @autoreleasepool {
        [lr_synth() stopSpeaking];
    }
}
