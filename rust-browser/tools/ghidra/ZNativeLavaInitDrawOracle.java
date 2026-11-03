// Read-only405350 recycled-capacity and406880 visible draw-queue oracle.
//The visible first callback does not move; C rand and sound sink isolated.
import ghidra.app.emulator.EmulatorHelper;
import java.io.*;
public class ZNativeLavaInitDrawOracle extends ZNativeLavaOracle{
 long read(EmulatorHelper e,long a,int n){byte[]b=e.readMemory(toAddr(a),n);long v=0;for(int i=0;i<n;i++)v|=(b[i]&255L)<<(i*8);return v;}
 public void run()throws Exception{try(PrintWriter out=new PrintWriter(getScriptArgs()[0])){
  for(int s=0;s<512;s++){EmulatorHelper e=new EmulatorHelper(currentProgram);try{setup(e,s,1);byte[]b=new byte[1920];for(int j=0;j<b.length;j++)b[j]=(byte)(j*17+s*13);e.writeMemory(toAddr(P),b);for(int i=0;i<32;i++)put(e,P+i*60,2,i<s%33?100+i:-1);put(e,0x4d18fe,1,s%5);put(e,0x62d5e0,2,(s>>3)&1);put(e,STACK+4,4,(s>>4)&1);put(e,STACK+8,4,2);put(e,STACK+12,4,s*7&255);put(e,STACK+16,4,s*11&255);invoke(e,0x405350,500-s,600+s);out.println("I,"+s+","+hash(e.readMemory(toAddr(P),1920)));}finally{e.dispose();}}
  for(int s=0;s<1024;s++){EmulatorHelper e=new EmulatorHelper(currentProgram);try{setup(e,s,1);long D=0x62dc50L,Q=0x628e18L;e.writeMemory(toAddr(D),new byte[16]);e.writeMemory(toAddr(Q),new byte[16]);put(e,0x62b3a4,4,D);put(e,0x62d5d8,4,Q);put(e,0x622cb8,2,12);put(e,0x622cba,2,20);put(e,0x6323a0,1,0);long a=P;put(e,a,2,150);put(e,a+2,2,160);put(e,a+8,2,6014);put(e,a+16,2,4318);put(e,a+32,2,4338);put(e,a+34,2,4354);put(e,a+40,1,s*17&255);put(e,a+41,1,s%6);put(e,a+42,1,2);put(e,a+47,1,s*13&255);put(e,a+46,1,s*7&255);put(e,a+48,1,new int[]{0,1,2,3,4,8,12,68}[s%8]);put(e,a+49,1,new int[]{0,8,16,32,64}[s/8%5]);put(e,a+28,2,new int[]{-1,0,2,34,256}[s/40%5]);invoke(e,0x406880,P,0);boolean q=read(e,D+6,2)==65535;long b=q?Q:D;int y=(short)read(e,b+2,2)+20;out.println("D,"+s+","+((short)read(e,b,2)+12)+","+y+","+(short)read(e,b+4,2)+","+(byte)read(e,b+6,1)+","+(q?read(e,b+7,1):0)+","+(q?read(e,b+9,1):0)+","+(byte)read(e,b+(q?14:7),1));}finally{e.dispose();}}
 }}
}
