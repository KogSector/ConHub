import { FC, HTMLAttributes } from "react";

interface HeroImageProps extends HTMLAttributes<HTMLImageElement> {
	src: string;
	alt: string;
}

export const HeroImage: FC<HeroImageProps> = ({
	src,
	alt,
	className,
	...props
}) => {
	return (
		<img
			src={src}
			alt={alt}
			className={className}
			{...props}
			width={1920}
			height={1080}
		/>
	);
};
