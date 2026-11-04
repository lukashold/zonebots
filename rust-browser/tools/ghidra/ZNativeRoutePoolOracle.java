// Execute59c60/52500/524b0/44d1a0 with the complete finite route arrays.
// No called route allocator/cleanup functions are stubbed. Output is private.
import ghidra.app.script.GhidraScript;
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeRoutePoolOracle extends GhidraScript{
 static final long RET=0x12345000L,SP=0x7fff0000L,U=0x5e30c8L,N=0x5eee28L,B=0x5f8428L,H=0x71000000L;
 void put(EmulatorHelper e,long a,int n,long v)throws Exception{e.writeMemoryValue(toAddr(a),n,v);}
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(8*i);return v;}
 static void write(byte[]a,int o,int n,long v){for(int i=0;i<n;i++)a[o+i]=(byte)(v>>(8*i));}
 static long word(byte[]a,int o,int n){long v=0;for(int i=0;i<n;i++)v|=(a[o+i]&255L)<<(8*i);return v;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){for(int mode=0;mode<4;mode++)for(int s=0;s<256;s++){
  EmulatorHelper e=new EmulatorHelper(currentProgram);try{
   e.writeMemory(toAddr(U),new byte[240*192]);e.writeMemory(toAddr(H),new byte[44]);e.writeMemory(toAddr(0x62dc08L),new byte[44]);
   byte[]nodes=new byte[14400*8];for(int i=0;i<14400;i++){write(nodes,i*8,2,i*7+s);write(nodes,i*8+2,2,i*11-s);}
   int free=new int[]{0,1,239,240,4799,4800,14399,14400}[s%8];if(free<14400)write(nodes,free*8+4,4,-1);
   write(nodes,500*8+4,4,N+501*8);write(nodes,501*8+4,4,0);write(nodes,5000*8+4,4,N+5001*8);write(nodes,5001*8+4,4,0);e.writeMemory(toAddr(N),nodes);
   int flags=(s*3&3)|32|64|((s&1)==0?4:0),role=new int[]{-1,0,1}[s%3];
   put(e,U+0x31,1,flags);put(e,U+0x5e,1,role);put(e,U+0x14,4,N+500*8);put(e,U+0xc,4,N+5000*8);put(e,U+8,4,(s&64)!=0?0:H);put(e,H+23,1,s*13);put(e,0x62dc08L+23,1,s*13);
   put(e,U+192+0x31,1,(s&32)!=0?4:0);put(e,U+192+0x14,4,(s&16)!=0?N+free*8:N+500*8);
   put(e,0x63288c,4,(s&8)!=0?U:U+192);put(e,0x6338dc,4,(s&4)!=0?0x43e4a0:0x40c810);put(e,0x4d02ba,1,s&2);
   long candidate=new long[]{0,0xffffffffL,N,B,B+8,N+4799*8,N-8}[s%7];
   e.writeRegister("ESP",SP);put(e,SP,4,RET);put(e,SP+4,4,800-s);e.writeRegister("ECX",U);e.writeRegister("EDX",mode==3?candidate:-300+s);e.setBreakpoint(toAddr(RET));
   if(!e.run(toAddr(new long[]{0x459c60,0x452500,0x4524b0,0x44d1a0}[mode]),null,monitor))throw new Exception(e.getLastError());
   byte[]after=e.readMemory(toAddr(N),14400*8);int hash=0x811c9dc5;for(int i=0;i<14400;i++){for(int v:new int[]{(short)word(after,i*8,2),(short)word(after,i*8+2,2),(int)word(after,i*8+4,4)})hash=(hash^v)*16777619;}
   out.println("{\"mode\":"+mode+",\"scenario\":"+s+",\"primary\":"+read(e,U+0x14,4)+",\"secondary\":"+read(e,U+0xc,4)+",\"flags\":"+read(e,U+0x31,1)+",\"hardware17\":"+read(e,(s&64)!=0?0x62dc08L+23:H+23,1)+",\"function\":"+read(e,0x6338dc,4)+",\"result\":"+(mode==0?e.readRegister("EAX").intValue()&255:0)+",\"hash\":"+(hash&0xffffffffL)+"}");
  }finally{e.dispose();}
 }}}
}
