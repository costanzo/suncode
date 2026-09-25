package ai.suncode.mobile

interface Platform {
    val name: String
}

expect fun getPlatform(): Platform