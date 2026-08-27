using System;
using System.IO;
using System.Security.Cryptography;

public class BootAudit {
    public static void Main() {
        string imgPath = @"target\x86_64-catalyst\debug\catalyst-kernel.img";
        Console.WriteLine("============================================================");
        Console.WriteLine("  CATALYST OS - BOOT SECTOR READ-ONLY AUDIT                 ");
        Console.WriteLine("============================================================");

        if (!File.Exists(imgPath)) {
            Console.WriteLine("ERROR: Image not found: " + imgPath);
            return;
        }

        byte[] bytes = File.ReadAllBytes(imgPath);
        Console.WriteLine("\n[1] File Size Check:");
        Console.WriteLine("    Expected: 10,978,304 bytes");
        Console.WriteLine("    Actual  : " + bytes.Length.ToString("N0") + " bytes (" + (bytes.Length == 10978304 ? "PASS" : "FAIL") + ")");

        Console.WriteLine("\n[2] SHA-256 Checksum Check:");
        string shaStr;
        using (SHA256 sha = SHA256.Create()) {
            shaStr = BitConverter.ToString(sha.ComputeHash(bytes)).Replace("-", "").ToUpper();
        }
        Console.WriteLine("    Expected: 298587EF635AB5C67429F55F7AACB1A29500FB9F727B0BC18F69299EC6CD3CAC");
        Console.WriteLine("    Actual  : " + shaStr + " (" + (shaStr == "298587EF635AB5C67429F55F7AACB1A29500FB9F727B0BC18F69299EC6CD3CAC" ? "PASS" : "FAIL") + ")");

        Console.WriteLine("\n[3] Sector 0 Boot Signature Check (Offset 510..511):");
        byte b510 = bytes[510];
        byte b511 = bytes[511];
        Console.WriteLine(string.Format("    Offset 0x1FE-0x1FF Magic: 0x{0:X2} 0x{1:X2}", b510, b511));
        bool sigValid = (b510 == 0x55 && b511 == 0xAA);
        Console.WriteLine("    BIOS Magic Signature: " + (sigValid ? "PASS (Valid 0x55AA BIOS Signature)" : "FAIL"));

        Console.WriteLine("\n[4] Bootstrap Entry Point (First 16 bytes):");
        string hex16 = "";
        for (int i = 0; i < 16; i++) hex16 += string.Format("{0:X2} ", bytes[i]);
        Console.WriteLine("    Opcodes: " + hex16);
        Console.WriteLine(string.Format("    Initial Instruction: 0x{0:X2} 0x{1:X2} (x86 Real Mode JMP/NOP Bootstrap)", bytes[0], bytes[1]));

        Console.WriteLine("\n[5] MBR Partition Table Entries (Offset 446..509):");
        for (int p = 0; p < 4; p++) {
            int off = 446 + (p * 16);
            string pHex = "";
            for (int k = 0; k < 16; k++) pHex += string.Format("{0:X2} ", bytes[off + k]);
            byte bootFlag = bytes[off];
            byte pType = bytes[off + 4];
            uint lba = BitConverter.ToUInt32(bytes, off + 8);
            uint count = BitConverter.ToUInt32(bytes, off + 12);
            Console.WriteLine(string.Format("    Partition {0} (0x{1:X3}): {2}", p + 1, off, pHex));
            Console.WriteLine(string.Format("      Boot Flag: 0x{0:X2} ({1}), Type: 0x{2:X2}, LBA: {3}, Sectors: {4}",
                bootFlag, (bootFlag == 0x80 ? "ACTIVE/BOOTABLE" : "INACTIVE"), pType, lba, count));
        }

        Console.WriteLine("\n============================================================");
        Console.WriteLine("  FINAL AUDIT RESULT: " + (bytes.Length == 10978304 && shaStr == "298587EF635AB5C67429F55F7AACB1A29500FB9F727B0BC18F69299EC6CD3CAC" && sigValid ? "ALL INVARIANTS PASS (BIOS/CSM READY)" : "AUDIT FAILED"));
        Console.WriteLine("============================================================");
    }
}
