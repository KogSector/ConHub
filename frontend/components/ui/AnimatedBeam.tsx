"use client";

import { cn } from "@/lib/utils";
import { motion, useInView } from "framer-motion";
import {
	FC,
	HTMLAttributes,
	ReactNode,
	useEffect,
	useRef,
	useState,
} from "react";

interface AnimatedBeamProps extends HTMLAttributes<HTMLDivElement> {
	children: ReactNode;
	duration?: number;
	delay?: number;
	path?: string;
	gradientId?: string;
	gradientColor?: string;
	gradientStopOpacity?: string;
}

export const AnimatedBeam: FC<AnimatedBeamProps> = ({
	children,
	className,
	duration = 10,
	delay = 0,
	path = "M 0 0 L 100 0",
	gradientId = "beam-gradient",
	gradientColor = "#ffaa40",
	gradientStopOpacity = "0",
	...props
}) => {
	const ref = useRef<HTMLDivElement>(null);
	const inView = useInView(ref, { once: true });
	const [animationStarted, setAnimationStarted] = useState(false);

	useEffect(() => {
		if (inView) {
			setAnimationStarted(true);
		}
	}, [inView]);

	return (
		<div
			ref={ref}
			className={cn(
				"relative flex w-full items-center justify-center",
				className,
			)}
			{...props}
		>
			<svg
				width="100%"
				height="100%"
				viewBox="0 0 100 100"
				preserveAspectRatio="none"
				className="absolute inset-0"
			>
				<defs>
					<linearGradient id={gradientId} gradientUnits="userSpaceOnUse">
						<stop stopColor={gradientColor} />
						<stop offset="1" stopColor={gradientColor} stopOpacity={gradientStopOpacity} />
					</linearGradient>
				</defs>
				<motion.path
					d={path}
					stroke={`url(#${gradientId})`}
					strokeWidth="2"
					fill="none"
					initial={{ pathLength: 0 }}
					animate={{
						pathLength: animationStarted ? 1 : 0,
					}}
					transition={{
						duration,
						delay,
						ease: "linear",
					}}
				/>
			</svg>
			{children}
		</div>
	);
};
