import os
import sys
import numpy as np
import pandas as pd

import splynters
from splynters import Splinter

import pyroaring
from pyroaring import BitMap

def load_csv_data(filepath):
    """
    Loads a text file containing comma-separated integers into a numpy array.

    This function is optimized for performance by reading the entire file at once
    and using numpy's fast parsing capabilities. It assumes the file contains
    a single line of comma-separated values.

    Args:
        filepath (str): The path to the input text file.

    Returns:
        np.ndarray: A sorted numpy array of unique uint32 integers.
    """
    print(f"Loading data from '{filepath}'...")
    
    try:
        with open(filepath, 'r') as f:
            content = f.read()
        
        # convert from string to array of numbers
        data = np.fromstring(content, dtype=np.uint32, sep=',')
        # data.sort()

    except FileNotFoundError:
        print(f"Error: File not found at '{filepath}'")
        return None
    except Exception as e:
        print(f"An error occurred: {e}")
        return None
        
    return data

def benchmark_directory(directory_path):
    """
    Benchmarks the compression sizes for all .txt files in a given directory.

    Args:
        directory_path (str): The path to the directory containing data files.

    Returns:
        list[tuple]: A list of tuples, where each tuple contains
                     (filename, uncompressed_size, roaring_size, splinter_size).
    """
    if not os.path.isdir(directory_path):
        print(f"Error: Directory not found at '{directory_path}'")
        return []

    results = []
    
    print(f"--- Starting Benchmark for directory: {directory_path} ---")

    for filename in sorted(os.listdir(directory_path)):
        if filename.endswith(".txt"):
            filepath = os.path.join(directory_path, filename)
            
            data = load_csv_data(filepath)
            
            if data is None:
                continue # Skip if loading failed

            # --- Calculate sizes ---
            uncompressed_size = data.nbytes

            roaring_bitmap = BitMap(data)
            roaring_size = roaring_bitmap.__sizeof__()

            splinter = Splinter.from_list(data.tolist())
            splinter_size = splinter.__sizeof__()

            print(f"Processed '{filename}': Uncompressed={uncompressed_size}, Roaring={roaring_size}, Splinter={splinter_size}")
            
            results.append((filename, uncompressed_size, roaring_size, splinter_size))

    return results


def run_benchmarks_on_all_subdirs(top_level_dir, output_dir="benchmark_results"):
    """
    Iterates through subdirectories of a top-level directory, runs benchmarks
    on each, and saves the results to separate CSV files.

    Args:
        top_level_dir (str): The path to the main directory containing dataset subdirectories.
        output_dir (str): The directory where result CSVs will be saved.
    """
    if not os.path.isdir(top_level_dir):
        print(f"Error: Top-level directory not found at '{top_level_dir}'")
        return

    # Create the output directory if it doesn't exist
    os.makedirs(output_dir, exist_ok=True)
    print(f"Results will be saved in '{output_dir}/'")

    # Iterate over all items in the top-level directory
    for subdir_name in sorted(os.listdir(top_level_dir)):
        subdir_path = os.path.join(top_level_dir, subdir_name)

        # Check if the item is a directory
        if os.path.isdir(subdir_path):
            print(f"\n{'='*20} Processing Dataset: {subdir_name} {'='*20}")
            
            # Run the benchmark on this specific subdirectory
            results = benchmark_directory(subdir_path)

            if results:
                # Construct a unique output filename for this dataset
                output_csv_path = os.path.join(output_dir, f"results_{subdir_name}.csv")
                
                # Convert results to a DataFrame
                df = pd.DataFrame(
                    results,
                    columns=["Filename", "Uncompressed (bytes)", "Roaring (bytes)", "Splinter (bytes)"]
                )
                
                # Calculate compression ratios
                df['Roaring Ratio'] = df['Uncompressed (bytes)'] / df['Roaring (bytes)']
                df['Splinter Ratio'] = df['Uncompressed (bytes)'] / df['Splinter (bytes)']
                
                # Save to CSV
                try:
                    df.to_csv(output_csv_path, index=False)
                    print(f"✅ Successfully saved results to '{output_csv_path}'")
                except Exception as e:
                    print(f"❌ Error saving results for {subdir_name}: {e}")
            else:
                print(f"ℹ️ No .txt files found or processed in '{subdir_name}'. Skipping.")

def main():
    """Main function to run the full benchmark suite."""
    if len(sys.argv) < 2:
        print("Usage: python run_all_benchmarks.py <path_to_top_level_dataset_directory>")
        print("\nExample: python run_all_benchmarks.py real-roaring-dataset")
        sys.exit(1)
        
    target_directory = sys.argv[1]
    run_benchmarks_on_all_subdirs(target_directory)
    print("\n--- All benchmarks complete. ---")


if __name__ == "__main__":
    main()

