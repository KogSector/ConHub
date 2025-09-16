import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { User, Settings, LogOut, CreditCard } from "lucide-react";
import Link from "next/link";

export function ProfileAvatar() {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" className="relative h-10 w-10 rounded-full focus-visible:ring-0 focus-visible:ring-offset-0">
          <Avatar className="h-10 w-10">
            <AvatarImage src="/placeholder.svg" alt="Profile" />
            <AvatarFallback>JD</AvatarFallback>
          </Avatar>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-64 mr-4 mt-6">
        <DropdownMenuLabel className="font-normal py-3 px-4">
          <div className="flex flex-col space-y-2">
            <p className="text-base font-medium leading-none">John Doe</p>
            <p className="text-sm leading-none text-muted-foreground">
              john.doe@example.com
            </p>
          </div>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuItem asChild className="py-3 px-4 text-base">
          <Link href="/dashboard/settings" className="cursor-pointer">
            <User className="mr-3 h-5 w-5" />
            <span>Profile</span>
          </Link>
        </DropdownMenuItem>
        <DropdownMenuItem asChild className="py-3 px-4 text-base">
          <Link href="/dashboard/settings?tab=billing" className="cursor-pointer">
            <CreditCard className="mr-3 h-5 w-5" />
            <span>Billing</span>
          </Link>
        </DropdownMenuItem>
        <DropdownMenuItem asChild className="py-3 px-4 text-base">
          <Link href="/dashboard/settings?tab=security" className="cursor-pointer">
            <Settings className="mr-3 h-5 w-5" />
            <span>Settings</span>
          </Link>
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem className="cursor-pointer py-3 px-4 text-base text-red-600 hover:bg-purple-100 hover:text-red-600 focus:bg-purple-100 focus:text-red-600">
          <LogOut className="mr-3 h-5 w-5" />
          <span>Log out</span>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}