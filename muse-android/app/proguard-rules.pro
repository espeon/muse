# kotlinx.serialization
-keepattributes *Annotation*, InnerClasses
-dontnote kotlinx.serialization.AnnotationsKt

-keepclassmembers class kotlinx.serialization.json.** {
    *** Companion;
}
-keepclasseswithmembers class kotlinx.serialization.json.** {
    kotlinx.serialization.KSerializer serializer(...);
}

-keep,includedescriptorclasses class vg.nat.muse.**$$serializer { *; }
-keepclassmembers class vg.nat.muse.** {
    *** Companion;
}
-keepclasseswithmembers class vg.nat.muse.** {
    kotlinx.serialization.KSerializer serializer(...);
}

# ktor
-dontwarn io.ktor.**
-keep class io.ktor.** { *; }

# media3
-dontwarn androidx.media3.**
-keep class androidx.media3.** { *; }

# ML Kit
-dontwarn com.google.mlkit.**
-keep class com.google.mlkit.** { *; }

# coil
-dontwarn coil.**
-keep class coil.** { *; }

# AndroidX security crypto
-keep class androidx.security.crypto.** { *; }
