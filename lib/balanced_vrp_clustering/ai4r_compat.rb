# Copyright © Cartoway
#
# Minimal local replacements for the former ai4r dependency.
# Kept under the Ai4r:: namespace for public API and Marshal fixture compatibility.
#
# frozen_string_literal: true

module Ai4r
  module Data
    # Declares configurable parameters (attr_accessor + docs hash).
    module Parameterizable
      module ClassMethods
        def get_parameters_info
          @_params_info_ || {}
        end

        def parameters_info(params_info)
          @_params_info_ = get_parameters_info.merge(params_info)
          params_info.each_key do |param|
            attr_accessor param unless method_defined?(param) || method_defined?("#{param}=")
          end
        end
      end

      def set_parameters(params)
        params.each do |key, val|
          public_send("#{key}=", val) if respond_to?("#{key}=")
        end
        self
      end

      def get_parameters
        params = {}
        self.class.get_parameters_info.each_key do |key|
          params[key] = send(key) if respond_to?(key)
        end
        params
      end

      def self.included(base)
        base.extend(ClassMethods)
      end
    end

    # Collection of data items used as clustering input / cluster containers.
    # Instance variable names must stay stable for Marshal.load of fixtures.
    class DataSet
      attr_reader :data_labels, :data_items

      def initialize(options = {})
        @data_labels = []
        @data_items = options[:data_items] || []
        set_data_labels(options[:data_labels]) if options[:data_labels]
        set_data_items(options[:data_items]) if options[:data_items]
      end

      def set_data_labels(labels)
        check_data_labels(labels)
        @data_labels = labels
        self
      end

      def set_data_items(items)
        check_data_items(items)
        @data_labels = default_data_labels(items) if @data_labels.empty?
        @data_items = items
        self
      end

      def num_attributes
        @data_items.empty? ? 0 : @data_items.first.size
      end

      def <<(data_item)
        if data_item.nil? || !data_item.is_a?(Enumerable) || data_item.empty?
          raise ArgumentError, 'Data must not be an non empty array.'
        elsif @data_items.empty?
          set_data_items([data_item])
        elsif data_item.length != num_attributes
          raise ArgumentError, 'Number of attributes do not match. ' \
                               "#{data_item.length} attributes provided, " \
                               "#{num_attributes} attributes expected."
        else
          @data_items << data_item
        end
      end

      protected

      def check_data_items(data_items)
        if !data_items || data_items.empty?
          raise ArgumentError, 'Examples data set must not be empty.'
        elsif !data_items.first.is_a?(Enumerable)
          raise ArgumentError, 'Unkown format for example data.'
        end

        attributes_num = data_items.first.length
        data_items.each_index do |index|
          next if data_items[index].length == attributes_num

          raise ArgumentError,
                'Quantity of attributes is inconsistent. ' \
                "The first item has #{attributes_num} attributes " \
                "and row #{index} has #{data_items[index].length} attributes"
        end
      end

      def check_data_labels(labels)
        return if @data_items.empty?
        return if labels.length == @data_items.first.length

        raise ArgumentError,
              'Number of labels and attributes do not match. ' \
              "#{labels.length} labels and " \
              "#{@data_items.first.length} attributes found."
      end

      def default_data_labels(data_items)
        data_labels = []
        data_items[0][0..-2].each_index do |i|
          data_labels[i] = "attribute_#{i + 1}"
        end
        data_labels[data_labels.length] = 'class_value'
        data_labels
      end
    end
  end

  module Clusterers
    # Thin base replacing ai4r's Clusterer/KMeans scaffolding.
    class Clusterer
      include Ai4r::Data::Parameterizable

      attr_reader :data_set, :number_of_clusters, :clusters, :centroids

      protected

      def get_min_index(array)
        min = array.first
        index = 0
        array.each_index do |i|
          x = array[i]
          if x < min
            min = x
            index = i
          end
        end
        index
      end
    end
  end
end
