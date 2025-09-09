import { cn } from "@/lib/utils";
import { FC, HTMLAttributes, ReactNode } from "react";

interface ShinyButtonProps extends HTMLAttributes<HTMLButtonElement> {
	children: ReactNode;
}

export const ShinyButton: FC<ShinyButtonProps> = ({
	children,
	className,
	...props
}) => {
	return (
		<button
			className={cn(
				"relative inline-flex w-fit min-w-[200px] items-center justify-center rounded-xl bg-white px-6 py-2 text-center font-medium text-black",
				"dark:bg-black dark:text-white",
				"animate-shine bg-[length:200%_100%] bg-[linear-gradient(110deg,transparent,45%,#ffffff,55%,transparent)]",
				className,
			)}
			{...props}
		>
			{children}
		</button>
	);
};
