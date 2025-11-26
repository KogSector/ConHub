import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/DropdownMenu";
import { User, LogOut, CreditCard } from "lucide-react";
import Link from "next/link";
import { useAuth } from "@/hooks/use-auth";

export function ProfileAvatar() {
  const { user, logout, isAuthenticated } = useAuth()

  const getInitial = (name: string) => {
    const trimmed = name?.trim()
    return trimmed ? trimmed.charAt(0).toUpperCase() : 'U'
  }

  if (!isAuthenticated || !user) {
    return null
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" className="relative h-10 w-10 rounded-full focus-visible:ring-0 focus-visible:ring-offset-0">
          <Avatar className="h-10 w-10">
            <AvatarFallback>{getInitial(user.name)}</AvatarFallback>
          </Avatar>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-64 mr-4 mt-6">
        <DropdownMenuLabel className="font-normal py-3 px-4">
          <div className="flex flex-col space-y-2">
            <p className="text-base font-medium leading-none">{user.name}</p>
            <p className="text-sm leading-none text-muted-foreground">
              {user.email}
            </p>
          </div>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuItem asChild className="py-3 px-4 text-base">
          <Link href="/account" className="cursor-pointer">
            <User className="mr-3 h-5 w-5" />
            <span>Account</span>
          </Link>
        </DropdownMenuItem>
        <DropdownMenuItem asChild className="py-3 px-4 text-base">
          <Link href="/billing" className="cursor-pointer">
            <CreditCard className="mr-3 h-5 w-5" />
            <span>Usage & Subs</span>
          </Link>
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem 
          className="cursor-pointer py-3 px-4 text-base text-red-600 hover:bg-purple-100 hover:text-red-600 focus:bg-purple-100 focus:text-red-600"
          onClick={logout}
        >
          <LogOut className="mr-3 h-5 w-5" />
          <span>Log out</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
