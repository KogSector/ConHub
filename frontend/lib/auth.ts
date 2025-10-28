import NextAuth, { NextAuthOptions, Session, User } from "next-auth"
import { JWT } from "next-auth/jwt"
import GithubProvider from "next-auth/providers/github"
import GoogleProvider from "next-auth/providers/google"

// Bitbucket OAuth provider configuration
const BitbucketProvider = (options: {
  clientId: string
  clientSecret: string
}) => ({
  id: "bitbucket",
  name: "Bitbucket",
  type: "oauth" as const,
  authorization: {
    url: "https://bitbucket.org/site/oauth2/authorize",
    params: { scope: "account email" }
  },
  token: "https://bitbucket.org/site/oauth2/access_token",
  userinfo: "https://api.bitbucket.org/2.0/user",
  clientId: options.clientId,
  clientSecret: options.clientSecret,
  profile(profile) {
    return {
      id: profile.uuid,
      name: profile.display_name,
      email: profile.email,
      image: profile.links?.avatar?.href,
    }
  },
})

export const authOptions: NextAuthOptions = {
  providers: [
    GithubProvider({
      clientId: process.env.GITHUB_CLIENT_ID || "",
      clientSecret: process.env.GITHUB_CLIENT_SECRET || "",
      authorization: {
        params: {
          scope: "read:user user:email",
        },
      },
    }),
    GoogleProvider({
      clientId: process.env.GOOGLE_CLIENT_ID || "",
      clientSecret: process.env.GOOGLE_CLIENT_SECRET || "",
      authorization: {
        params: {
          scope: "openid email profile",
          prompt: "consent",
          access_type: "offline",
          response_type: "code",
        },
      },
    }),
    BitbucketProvider({
      clientId: process.env.BITBUCKET_CLIENT_ID || "",
      clientSecret: process.env.BITBUCKET_CLIENT_SECRET || "",
    }),
  ],
  
  callbacks: {
    async signIn({ user, account, profile }) {
      // Allow sign in for all OAuth providers
      if (account?.provider && user.email) {
        try {
          // Call Rust backend to create or update user with social connection
          const backendUrl = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001"
          const response = await fetch(`${backendUrl}/api/auth/oauth/callback`, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
              provider: account.provider,
              provider_user_id: account.providerAccountId,
              email: user.email,
              name: user.name,
              avatar_url: user.image,
              access_token: account.access_token,
              refresh_token: account.refresh_token,
              expires_at: account.expires_at,
              scope: account.scope,
            }),
          })

          if (!response.ok) {
            console.error("Failed to sync user with backend:", await response.text())
            return false
          }

          const userData = await response.json()
          // Store backend user ID for later use
          user.id = userData.user_id || user.id
        } catch (error) {
          console.error("Error syncing with backend:", error)
          return false
        }
      }
      return true
    },

    async jwt({ token, user, account }) {
      // Initial sign in
      if (account && user) {
        return {
          ...token,
          accessToken: account.access_token,
          refreshToken: account.refresh_token,
          accessTokenExpires: account.expires_at ? account.expires_at * 1000 : 0,
          provider: account.provider,
          userId: user.id,
        }
      }

      // Return previous token if the access token has not expired yet
      if (Date.now() < (token.accessTokenExpires as number)) {
        return token
      }

      // Access token has expired, try to refresh it
      // Note: Refresh logic can be implemented here if needed
      return token
    },

    async session({ session, token }: { session: Session; token: JWT }) {
      // Send properties to the client
      if (token) {
        session.accessToken = token.accessToken as string
        session.provider = token.provider as string
        session.userId = token.userId as string
      }
      return session
    },
  },

  pages: {
    signIn: "/auth/signin",
    error: "/auth/error",
  },

  session: {
    strategy: "jwt",
    maxAge: 2 * 60 * 60, // 2 hours (matching existing auth timeout)
  },

  secret: process.env.NEXTAUTH_SECRET,
}

export default NextAuth(authOptions)
