import java.io.*;
import java.util.*;
import java.text.*;
import java.math.*;

public class Main {
    static class Solver {
        int solve(int a, int b) {
            return a + b;
        }
    }

    public static void main(String[] args) {
        Scanner in = new Scanner(System.in);
        int a = in.nextInt();
        int b = in.nextInt();
        int sum = new Solver().solve(a, b);
        System.out.println(sum);
    }
}
