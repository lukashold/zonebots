// Read-only46ad70/46ad10 portrait clock, synthetic phrase words.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativePortraitClockOracle extends GhidraScript{
 static final long S=0x7fff0000L,R=0x12345000L,P=0x1000000L;
 void put(EmulatorHelper e,long a,int n,long v){e.writeMemoryValue(toAddr(a),n,v);}
 int read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);int v=0;for(int i=0;i<n;i++)v|=(b[i]&255)<<(i*8);return v;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){EmulatorHelper e=new EmulatorHelper(currentProgram);try{e.setBreakpoint(toAddr(R));for(int scenario=0;scenario<256;scenario++){int row=scenario%64,length=scenario%181;put(e,0x64c8a4,4,P);for(int f=0;f<256;f++)put(e,P+row*552+40+f*2,2,(f*40503+scenario*17)&65535);put(e,0x49c644,2,row);put(e,0x6809a4,2,length);put(e,0x6809a8,2,0);put(e,0x5849cb,1,3);put(e,0x64f7d4,2,0);put(e,0x66656c,2,0);for(int tick=0;tick<64;tick++){e.writeRegister("ESP",S);put(e,S,4,R);if(!e.run(toAddr(0x46ad70),null,monitor))throw new Exception(e.getLastError());out.println(scenario+","+tick+","+read(e,0x6809a8,2));}}}finally{e.dispose();}}}
}
