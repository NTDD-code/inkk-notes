Add-Type -AssemblyName System.Drawing

function Convert-PngToIco {
    param(
        [string]$PngPath,
        [string]$IcoPath,
        [int[]]$Sizes = @(256, 128, 64, 48, 32, 16)
    )

    $srcImg = [System.Drawing.Bitmap]::FromFile($PngPath)
    $images = @()

    foreach ($size in $Sizes) {
        $bmp = New-Object System.Drawing.Bitmap($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
        $g = [System.Drawing.Graphics]::FromImage($bmp)
        $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
        $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
        $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
        $g.DrawImage($srcImg, 0, 0, $size, $size)
        $g.Dispose()

        $ms = New-Object System.IO.MemoryStream
        if ($size -ge 64) {
            # PNG encoded
            $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
            $isPng = $true
        } else {
            # Standard DIB encoded (compatible with Inno Setup / older Windows dialogs)
            $isPng = $false
            # Write BITMAPINFOHEADER (40 bytes)
            $bw = New-Object System.IO.BinaryWriter($ms)
            $bw.Write([uint32]40)              # biSize
            $bw.Write([int32]$size)            # biWidth
            $bw.Write([int32]($size * 2))      # biHeight (XOR + AND mask)
            $bw.Write([uint16]1)               # biPlanes
            $bw.Write([uint16]32)              # biBitCount
            $bw.Write([uint32]0)               # biCompression = BI_RGB
            $bw.Write([uint32]($size * $size * 4)) # biSizeImage
            $bw.Write([int32]0)                # biXPelsPerMeter
            $bw.Write([int32]0)                # biYPelsPerMeter
            $bw.Write([uint32]0)               # biClrUsed
            $bw.Write([uint32]0)               # biClrImportant

            # Pixel data (bottom-up BGRA)
            for ($y = $size - 1; $y -ge 0; $y--) {
                for ($x = 0; $x -lt $size; $x++) {
                    $p = $bmp.GetPixel($x, $y)
                    $bw.Write([byte]$p.B)
                    $bw.Write([byte]$p.G)
                    $bw.Write([byte]$p.R)
                    $bw.Write([byte]$p.A)
                }
            }
            # AND mask (1 bit per pixel, row padded to 32 bits, all 0)
            $rowBytes = [Math]::Ceiling($size / 32.0) * 4
            $andRow = New-Object byte[] $rowBytes
            for ($y = 0; $y -lt $size; $y++) {
                $bw.Write($andRow, 0, $rowBytes)
            }
        }
        $bmp.Dispose()
        $images += @{
            Size = $size
            Data = $ms.ToArray()
            IsPng = $isPng
        }
        $ms.Dispose()
    }
    $srcImg.Dispose()

    # Write ICO file
    $fs = [System.IO.File]::Create($IcoPath)
    $w = New-Object System.IO.BinaryWriter($fs)

    # ICONDIR
    $w.Write([uint16]0) # Reserved
    $w.Write([uint16]1) # Type: 1 = ICO
    $w.Write([uint16]$images.Count) # Count

    $offset = 6 + ($images.Count * 16)
    foreach ($img in $images) {
        $s = $img.Size
        $wByte = if ($s -eq 256) { [byte]0 } else { [byte]$s }
        $hByte = if ($s -eq 256) { [byte]0 } else { [byte]$s }
        $w.Write($wByte) # Width
        $w.Write($hByte) # Height
        $w.Write([byte]0) # Color count
        $w.Write([byte]0) # Reserved
        $w.Write([uint16]1) # Planes
        $w.Write([uint16]32) # Bit count
        $w.Write([uint32]$img.Data.Length) # Bytes in res
        $w.Write([uint32]$offset) # Offset
        $offset += $img.Data.Length
    }

    # Write Image Data
    foreach ($img in $images) {
        $w.Write($img.Data, 0, $img.Data.Length)
    }

    $w.Flush()
    $w.Close()
    $fs.Close()
    Write-Host "Generated $IcoPath successfully! Total size: $( (Get-Item $IcoPath).Length ) bytes"
}

Convert-PngToIco -PngPath "assets\icon.png" -IcoPath "assets\icon.ico"
